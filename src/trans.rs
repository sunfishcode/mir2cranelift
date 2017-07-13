extern crate cretonne;
extern crate cton_frontend;

use error::*;
use rustc::mir::{Mir, Local};
use rustc::mir::{UnOp, BinOp, Literal, Lvalue, Operand, ProjectionElem, Rvalue, AggregateKind,
                 CastKind, StatementKind, TerminatorKind};
use rustc::dep_graph::DepNode;
use rustc::middle::const_val::ConstVal;
use rustc_const_math::{ConstInt, ConstIsize};
use rustc::ty::{self, TyCtxt, Ty, FnSig};
use rustc::ty::layout::{self, Layout, Size};
use rustc::ty::subst::Substs;
use rustc::hir::intravisit::{self, Visitor, FnKind, NestedVisitorMap};
use rustc::hir::{FnDecl, BodyId};
use rustc::hir::def_id::DefId;
use rustc::traits::Reveal;
use syntax::ast::{NodeId, IntTy, UintTy, FloatTy};
use syntax::codemap::Span;
use std::ptr;
use std::collections::HashMap;
use std::cell::RefCell;
use monomorphize;
use traits;
use rustc_data_structures::indexed_vec::Idx;
use self::cretonne::ir::condcodes::IntCC;
use self::cretonne::ir::InstBuilder;
use std::u32;

#[derive(Debug, Clone)]
pub struct Mir2CretonneTransOptions {
    pub optimize: bool,
    pub print: bool,
    pub binary_output_path: Option<String>,
}

impl Mir2CretonneTransOptions {
    pub fn new() -> Mir2CretonneTransOptions {
        Mir2CretonneTransOptions {
            optimize: false,
            print: true,
            binary_output_path: None,
        }
    }
}

fn visit_krate<'g, 'tcx>(tcx: TyCtxt<'g, 'tcx, 'tcx>,
                         entry_fn: Option<NodeId>)
                         -> Vec<cretonne::ir::Function> {
    let mut context: CretonneModuleCtxt = CretonneModuleCtxt::new(tcx, entry_fn);
    tcx.hir
        .krate()
        .visit_all_item_likes(&mut context.as_deep_visitor());
    context.functions
}

pub fn trans_crate<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                             entry_fn: Option<NodeId>,
                             options: &Mir2CretonneTransOptions)
                             -> Result<()> {

    let _ignore = tcx.dep_graph.in_ignore();

    let functions = visit_krate(tcx, entry_fn);

    // TODO: Run the Cretonne verifier.

    // TODO: Run Cretonne optimization passes.

    if options.print {
        panic!("Unimplemented: print the functions");
    }

    // TODO: Emit output.

    Ok(())
}

struct CretonneModuleCtxt<'b, 'gcx: 'b + 'tcx, 'tcx: 'b> {
    tcx: TyCtxt<'b, 'gcx, 'tcx>,
    entry_fn: Option<NodeId>,
    fun_types: HashMap<ty::FnSig<'gcx>, cretonne::ir::SigRef>,
    fun_names: HashMap<(DefId, ty::FnSig<'gcx>), String>,
    functions: Vec<cretonne::ir::Function>,
}

impl<'c, 'gcx: 'c + 'tcx, 'tcx: 'c> CretonneModuleCtxt<'c, 'gcx, 'tcx> {
    fn new(tcx: TyCtxt<'c, 'gcx, 'tcx>,
           entry_fn: Option<NodeId>)
           -> CretonneModuleCtxt<'c, 'gcx, 'tcx> {
        CretonneModuleCtxt {
            tcx: tcx,
            entry_fn: entry_fn,
            fun_types: HashMap::new(),
            fun_names: HashMap::new(),
            functions: Vec::new(),
        }
    }
}

impl<'e, 'tcx: 'e, 'h> Visitor<'h> for CretonneModuleCtxt<'e, 'tcx, 'tcx> {
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'h> {
        NestedVisitorMap::None
    }

    fn visit_fn(&mut self, fk: FnKind<'h>, fd: &'h FnDecl, b: BodyId, s: Span, id: NodeId) {
        let did = self.tcx.hir.local_def_id(id);

        let generics = self.tcx.generics_of(did);

        // don't translate generic functions yet
        if generics.types.len() + generics.parent_types as usize > 0 {
            return;
        }

        let mir = self.tcx.optimized_mir(did);

        let sig = self.tcx.fn_sig(did);
        let sig = sig.skip_binder();
        let mut func = cretonne::ir::Function::new();
        let mut il_builder = cton_frontend::ILBuilder::new();
        let mut func_builder = cton_frontend::FunctionBuilder::new(&mut func, &mut il_builder);
        {
            let mut ctxt = CretonneFnCtxt {
                tcx: self.tcx,
                mir: mir,
                did: did,
                sig: sig,
                builder: &mut func_builder,
                entry_fn: self.entry_fn,
                fun_types: &mut self.fun_types,
                fun_names: &mut self.fun_names,
                checked_op_local: None,
                var_map: Vec::new(),
                temp_map: Vec::new(),
                ret_var: None,
            };

            ctxt.trans();
        }

        intravisit::walk_fn(self, fk, fd, b, s, id)
    }
}

// An opaque reference to local variable in Rust.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Variable(u32);
impl cretonne::entity_ref::EntityRef for Variable {
    fn new(index: usize) -> Self {
        assert!(index < (u32::MAX as usize));
        Variable(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}
impl Default for Variable {
    fn default() -> Variable {
        Variable(u32::MAX)
    }
}

struct CretonneFnCtxt<'f: 'g, 'g, 'd, 'gcx: 'd + 'tcx, 'tcx: 'd> {
    tcx: TyCtxt<'d, 'gcx, 'tcx>,
    mir: &'d Mir<'tcx>,
    did: DefId,
    sig: &'d FnSig<'gcx>,
    builder: &'g mut cton_frontend::FunctionBuilder<'f, Variable>,
    entry_fn: Option<NodeId>,
    fun_types: &'d mut HashMap<ty::FnSig<'gcx>, cretonne::ir::SigRef>,
    fun_names: &'d mut HashMap<(DefId, ty::FnSig<'gcx>), String>,
    checked_op_local: Option<u32>,
    var_map: Vec<Option<usize>>,
    temp_map: Vec<Option<usize>>,
    ret_var: Option<usize>,
}

impl<'f: 'g, 'g, 'd, 'gcx: 'd + 'tcx, 'tcx: 'd> CretonneFnCtxt<'f, 'g, 'd, 'gcx, 'tcx> {
    fn num_args(&self) -> usize {
        self.sig.inputs().len()
    }

    fn get_local_index(&self, i: usize) -> Option<usize> {
        debug!("fetching local {:?}", i);
        debug!("  vars: {:?}", self.var_map);
        debug!("  temps: {:?}", self.temp_map);
        if i == 0 {
            debug!("returning retvar");
            return self.ret_var;
        }
        let i = i - 1;
        if i < self.num_args() {
            debug!("returning function arg {}", i);
            return Some(i);
        }
        let i = i - self.num_args();
        if i < self.var_map.len() {
            debug!("returning {}th local: {:?}", i, self.var_map[i]);
            return self.var_map[i];
        }
        let i = i - self.var_map.len();
        assert!(i < self.temp_map.len());
        debug!("returning {}th temp: {:?}", i, self.temp_map[i]);
        self.temp_map[i]
    }
}

impl<'f: 'g, 'g, 'd, 'tcx: 'd> CretonneFnCtxt<'f, 'g, 'd, 'tcx, 'tcx> {
    /// This is the main entry point for MIR->cretonne fn translation
    fn trans(&mut self) {
        let mir = self.mir;

        // Maintain a cache of translated monomorphizations and bail
        // if we've already seen this one.
        use std::collections::hash_map::Entry::*;
        match self.fun_names.entry((self.did, *self.sig)) {
            Occupied(_) => return,
            Vacant(entry) => {
                let fn_name = sanitize_symbol(&self.tcx.item_path_str(self.did));
                entry.insert(fn_name);
            }
        }

        debug!("translating fn {:?}", self.tcx.item_path_str(self.did));

        // Translate arg and ret tys to cretonne
        for ty in self.sig.inputs() {
            if let Some(cton_ty) = rust_ty_to_cretonne(ty) {
                panic!("Unimplemented: function arguments");
            }
        }
        let mut needs_ret_var = false;
        let ret_ty = self.sig.output();
        debug!("ret_ty is {:?}", ret_ty);
        let cretonne_ret = rust_ty_to_cretonne(ret_ty);
        needs_ret_var = cretonne_ret.is_some();
        debug!("needs_ret_var = {:?}", needs_ret_var);

        // Create the wasm vars.
        // Params and vars form the list of locals, both sharing the same index space.

        // TODO: Use mir.local_decls directly rather than the two iterators.
        for mir_var in mir.vars_iter() {
            debug!("adding local {:?}: {:?}",
                   mir_var,
                   mir.local_decls[mir_var].ty);
            match rust_ty_to_cretonne(mir.local_decls[mir_var].ty) {
                Some(cton_ty) => {
                    panic!("Unimplemented: local variables");
                }
                None => self.var_map.push(None),
            }
        }

        for mir_var in mir.temps_iter() {
            debug!("adding temp {:?}", mir_var);
            panic!("Unimplemented: temporary variables");
        }

        if needs_ret_var {
            debug!("adding ret var");
            panic!("Unimplemented: return variables");
        }

        debug!("{} MIR basic blocks to translate", mir.basic_blocks().len());

        for (i, bb) in mir.basic_blocks().iter().enumerate() {
            debug!("bb{}: {:#?}", i, bb);

            let mut cretonne_stmts = Vec::new();
            for stmt in &bb.statements {
                match stmt.kind {
                    StatementKind::Assign(ref lvalue, ref rvalue) => {
                        self.trans_assignment(lvalue, rvalue, &mut cretonne_stmts);
                    }
                    StatementKind::StorageLive(_) => {}
                    StatementKind::StorageDead(_) => {}
                    _ => panic!("{:?}", stmt.kind),
                }
            }

            let block_kind = CretonneBlockKind::Default;

            match bb.terminator().kind {
                TerminatorKind::Return => {
                    // TODO: Emit function epilogue, if necessary.
                    debug!("emitting Return from fn {:?}",
                           self.tcx.item_path_str(self.did));
                    if ret_ty.is_nil() {
                        self.builder.ins().return_(&[]);
                    } else {
                        // Local 0 is guaranteed to be return pointer
                        let v = self.trans_operand(&Operand::Consume(Lvalue::Local(Local::new(0))));
                        self.builder.ins().return_(&[v]);
                    }
                }
                TerminatorKind::Call {
                    ref func,
                    ref args,
                    ref destination,
                    ..
                } => {
                    panic!("Unimplemented: terminator calls");
                }
                TerminatorKind::Goto { ref target } => {
                    debug!("emitting Branch for Goto, from bb{} to bb{}",
                           i,
                           target.index());
                    panic!("Unimplemented: Goto");
                }
                TerminatorKind::Assert { ref target, .. } => {
                    // TODO: An assert is not a GOTO!!!
                    // Fix this!
                    debug!("emitting Branch for Goto, from bb{} to bb{}",
                           i,
                           target.index());
                    panic!("Unimplemented: Assert");
                }
                _ => (),
            }
        }

        if !self.fun_types.contains_key(self.sig) {
            let name = format!("rustfn-{}-{}", self.did.krate, self.did.index.as_u32());
            panic!("Unimplemented: declare function type");
            /*self.fun_types.insert(*self.sig, ty);*/
        }

        let nid = self.tcx.hir.as_local_node_id(self.did).expect("");

        if Some(self.did) == self.tcx.lang_items.panic_fn() {
            // TODO: when it's possible to print characters or interact with the environment,
            //       also handle #[lang = "panic_fmt"] to support panic messages
            debug!("emitting Unreachable function for panic lang item");
            panic!("Unimplemented: panic lang item");
        } else {
            // Create the function prologue
            // TODO: the epilogue and prologue are not always necessary
            debug!("emitting function prologue");
            panic!("Unimplemented: function prologue");
        }

        debug!("done translating fn {:?}\n",
               self.tcx.item_path_str(self.did));
    }

    fn trans_assignment(&mut self,
                        lvalue: &Lvalue<'tcx>,
                        rvalue: &Rvalue<'tcx>,
                        statements: &mut Vec<cretonne::ir::Value>) {
        let mir = self.mir;

        let dest = match self.trans_lval(lvalue) {
            Some(dest) => dest,
            None => {
                // TODO: the rvalue may have some effects that we need to preserve. For example,
                // reading from memory can cause a fault.
                debug!("trans_assignment lval is unit: {:?} = {:?}; skipping",
                       lvalue,
                       rvalue);
                return;
            }
        };
        let dest_ty = lvalue.ty(&*mir, self.tcx).to_ty(self.tcx);

        let dest_layout = self.type_layout(dest_ty);

        match *rvalue {
            Rvalue::Use(ref operand) => {
                let src = self.trans_operand(operand);
                let statement = match dest.offset {
                    Some(offset) => {
                        debug!("emitting Store + GetLocal({}) for Assign Use '{:?} = {:?}'",
                               dest.index,
                               lvalue,
                               rvalue);
                        let ptr = self.builder.use_var(Variable(dest.index));
                        // TODO: match on the dest_ty to know how many bytes to write, not just
                        // i32s
                        panic!("Unimplemented: rvalues");
                        /*
                        CretonneStore(self.func.module.module,
                                      4,
                                      offset,
                                      0,
                                      ptr,
                                      src,
                                      CretonneInt32())
                        */
                    }
                    None => {
                        debug!("emitting SetLocal({}) for Assign Use '{:?} = {:?}'",
                               dest.index,
                               lvalue,
                               rvalue);
                        panic!("Unimplemented: set_local for assign use");
                        /*
                        CretonneSetLocal(self.func.module.module, dest.index, src)
                        */
                    }
                };
                statements.push(statement);
            }

            Rvalue::UnaryOp(ref op, ref operand) => {
                let mut operand = self.trans_operand(operand);
                operand = match *op {
                    UnOp::Not => self.builder.ins().icmp_imm(IntCC::Equal, operand, 0),
                    _ => panic!("unimplemented UnOp: {:?}", op),
                };
                self.builder.def_var(Variable(dest.index), operand);
            }

            Rvalue::BinaryOp(ref op, ref left, ref right) => {
                let left = self.trans_operand(left);
                let right = self.trans_operand(right);

                // TODO: match on dest_ty.sty to implement binary ops for other types than just
                // integers
                // TODO: check if the dest_layout is signed or not (CEnum, etc)
                // TODO: comparisons are signed only for now, so implement unsigned ones
                let op = match *op {
                    BinOp::Add => self.builder.ins().iadd(left, right),
                    BinOp::Sub => self.builder.ins().isub(left, right),
                    BinOp::Mul => self.builder.ins().imul(left, right),
                    BinOp::Div => self.builder.ins().sdiv(left, right),
                    BinOp::BitAnd => self.builder.ins().band(left, right),
                    BinOp::BitOr => self.builder.ins().bor(left, right),
                    BinOp::BitXor => self.builder.ins().bxor(left, right),
                    BinOp::Eq => self.builder.ins().icmp(IntCC::Equal, left, right),
                    BinOp::Ne => self.builder.ins().icmp(IntCC::NotEqual, left, right),
                    BinOp::Lt => self.builder.ins().icmp(IntCC::SignedLessThan, left, right),
                    BinOp::Le => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, left, right),
                    BinOp::Gt => self.builder.ins().icmp(IntCC::SignedGreaterThan, left, right),
                    BinOp::Ge => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left, right),
                    _ => panic!("unimplemented BinOp: {:?}", op),
                };

                match dest.offset {
                    Some(offset) => {
                        debug!("emitting Store + GetLocal({}) for Assign BinaryOp '{:?} = \
                                {:?}'",
                               dest.index,
                               lvalue,
                               rvalue);
                        let ptr = self.builder.use_var(Variable(dest.index));
                        // TODO: Set the trap/align flags.
                        let memflags = cretonne::ir::MemFlags::new();
                        let memoffset = cretonne::ir::immediates::Offset32::new(offset as i32);
                        // TODO: match on the dest_ty to know how many bytes to write, not just
                        // i32s
                        self.builder.ins().store(memflags, op, ptr, memoffset);
                    }
                    None => {
                        debug!("emitting SetLocal({}) for Assign BinaryOp '{:?} = {:?}'",
                               dest.index,
                               lvalue,
                               rvalue);
                        self.builder.def_var(Variable(dest.index), op);
                    }
                }
            }

            Rvalue::CheckedBinaryOp(ref op, ref left, ref right) => {
                panic!("Unimplemented: Checked binary op");
            }

            Rvalue::Ref(_, _, ref lvalue) => {
                // TODO: for shared refs only ?
                // TODO: works for refs to "our stack", but not the locals on the wasm stack yet
                let expr = self.trans_operand(&Operand::Consume(lvalue.clone()));
                debug!("emitting SetLocal({}) for Assign Ref '{:?} = {:?}'",
                       dest.index,
                       lvalue,
                       rvalue);
                self.builder.def_var(Variable(dest.index), expr);
            }

            Rvalue::Aggregate(ref kind, ref operands) => {
                panic!("Unimplemented Rvalue::Aggregate");
            }

            Rvalue::Cast(ref kind, ref operand, _) => {
                if dest.offset.is_some() {
                    panic!("unimplemented '{:?}' Cast with offset", kind);
                }

                match *kind {
                    CastKind::Misc => {
                        let src = self.trans_operand(operand);
                        let src_ty = operand.ty(&*mir, self.tcx);
                        let src_layout = self.type_layout(src_ty);

                        // TODO: handle more of the casts (miri doesn't really handle every Misc
                        // cast either right now)
                        match (src_layout, &dest_ty.sty) {
                            (&Layout::Scalar { .. }, &ty::TyInt(_)) |
                            (&Layout::Scalar { .. }, &ty::TyUint(_)) => {
                                debug!("emitting SetLocal({}) for Scalar Cast Assign '{:?} = \
                                        {:?}'",
                                       dest.index,
                                       lvalue,
                                       rvalue);
                                self.builder.def_var(Variable(dest.index), src);
                            }
                            (&Layout::CEnum { .. }, &ty::TyInt(_)) |
                            (&Layout::CEnum { .. }, &ty::TyUint(_)) => {
                                debug!("emitting SetLocal({}) for CEnum Cast Assign '{:?} = {:?}'",
                                       dest.index,
                                       lvalue,
                                       rvalue);
                                self.builder.def_var(Variable(dest.index), src);
                            }
                            _ => {
                                panic!("unimplemented '{:?}' Cast '{:?} = {:?}', for {:?} to {:?}",
                                       kind,
                                       lvalue,
                                       rvalue,
                                       src_layout,
                                       dest_ty.sty)
                            }
                        }
                    }
                    _ => {
                        panic!("unimplemented '{:?}' Cast '{:?} = {:?}'",
                               kind,
                               lvalue,
                               rvalue)
                    }
                }
            }

            _ => panic!("unimplemented Assign '{:?} = {:?}'", lvalue, rvalue),
        }
    }

    // TODO this function changed from being passed offsets-after-field to offsets-of-field...
    // but I suspect it still does the right thing - emit a store for every field.
    // Did it miss the first field and emit after the last field of the struct before?
    fn emit_assign_fields<I>(&mut self,
                             offsets: I,
                             operands: &[Operand<'tcx>],
                             statements: &mut Vec<cretonne::ir::Value>)
        where I: IntoIterator<Item = u64>
    {
        panic!("Unimplemented: assign_fields");
        /*
        for (offset, operand) in offsets.into_iter().zip(operands) {
            // let operand_ty = mir.operand_ty(*self.tcx, operand);
            // TODO: match on the operand_ty to know how many bytes to store, not just i32s
            let src = self.trans_operand(operand);
            let write_field = CretonneStore(self.func.module.module,
                                            4,
                                            offset as u32,
                                            0,
                                            read_sp,
                                            src,
                                            CretonneInt32());
            statements.push(write_field);
        }
        */
    }

    fn trans_lval(&mut self, lvalue: &Lvalue<'tcx>) -> Option<CretonneLvalue> {
        let mir = self.mir;

        debug!("translating lval: {:?}", lvalue);

        let i = match *lvalue {
            Lvalue::Local(i) => {
                match self.get_local_index(i.index()) {
                    Some(i) => i as u32,
                    None => return None,
                }
            }
            Lvalue::Projection(ref projection) => {
                let base = match self.trans_lval(&projection.base) {
                    Some(base) => base,
                    None => return None,
                };
                let base_ty = projection.base.ty(&*mir, self.tcx).to_ty(self.tcx);
                let base_layout = self.type_layout(base_ty);

                match projection.elem {
                    ProjectionElem::Deref => {
                        if base.offset.is_none() {
                            return Some(CretonneLvalue::new(base.index, None, LvalueExtra::None));
                        }
                        panic!("unimplemented Deref {:?}", lvalue);
                    }
                    ProjectionElem::Field(ref field, _) => {
                        let variant = match *base_layout {
                            Layout::Univariant { ref variant, .. } => variant,
                            Layout::General { ref variants, .. } => {
                                if let LvalueExtra::DowncastVariant(variant_idx) = base.extra {
                                    &variants[variant_idx]
                                } else {
                                    panic!("field access on enum had no variant index");
                                }
                            }
                            _ => panic!("unimplemented Field Projection: {:?}", projection),
                        };

                        let offset = variant.offsets[field.index()].bytes() as u32;
                        return Some(CretonneLvalue::new(base.index,
                                                        base.offset,
                                                        LvalueExtra::None)
                                            .offset(offset));
                    }
                    ProjectionElem::Downcast(_, variant) => {
                        match *base_layout {
                            Layout::General { discr, .. } => {
                                assert!(base.offset.is_none(),
                                        "unimplemented Downcast Projection with offset");

                                let offset = discr.size().bytes() as u32;
                                return Some(
                                    CretonneLvalue::new(base.index, Some(offset),
                                                        LvalueExtra::DowncastVariant(variant)));
                            }
                            _ => panic!("unimplemented Downcast Projection: {:?}", projection),
                        }
                    }
                    _ => panic!("unimplemented Projection: {:?}", projection),
                }
            }
            _ => panic!("unimplemented Lvalue: {:?}", lvalue),
        };

        Some(CretonneLvalue::new(i as CretonneIndex, None, LvalueExtra::None))
    }

    fn trans_operand(&mut self, operand: &Operand<'tcx>) -> cretonne::ir::Value {
        let mir = self.mir;

        match *operand {
            Operand::Consume(ref lvalue) => {
                let cretonne_lvalue = match self.trans_lval(lvalue) {
                    Some(lval) => lval,
                    None => {
                        debug!("operand lval is unit: {:?}", operand);
                        panic!("Unimplemented: unit lvalues");
                    }
                };
                let lval_ty = lvalue.ty(&*mir, self.tcx);
                let t = lval_ty.to_ty(self.tcx);
                let t = rust_ty_to_cretonne(t);

                match cretonne_lvalue.offset {
                    Some(offset) => {
                        debug!("emitting GetLocal({}) + Load for '{:?}'",
                               cretonne_lvalue.index,
                               lvalue);
                        let ptr = self.builder.use_var(Variable(cretonne_lvalue.index));
                        // TODO: match on the field ty to know how many bytes to read, not just
                        // i32s
                        // TODO: Set the trap/align flags.
                        let memflags = cretonne::ir::MemFlags::new();
                        let memoffset = cretonne::ir::immediates::Offset32::new(offset as i32);
                        self.builder.ins().load(cretonne::ir::types::I32, memflags, ptr, memoffset)
                    }
                    None => {
                        // debug!("emitting GetLocal for '{:?}'", lvalue);
                        self.builder.use_var(Variable(cretonne_lvalue.index))
                    }
                }
            }
            Operand::Constant(ref c) => {
                match c.literal {
                    Literal::Value { ref value } => {
                        // TODO: handle more Rust types here
                        match *value {
                            ConstVal::Integral(ConstInt::Isize(ConstIsize::Is32(val))) |
                            ConstVal::Integral(ConstInt::I32(val)) => self.builder.ins().iconst(cretonne::ir::types::I32, val as i64),
                            // TODO: Since we're at the wasm32 stage, and until wasm64, it's
                            // probably best if isize is always i32 ?
                            ConstVal::Integral(ConstInt::Isize(ConstIsize::Is64(val))) => {
                                self.builder.ins().iconst(cretonne::ir::types::I64, val)
                            }
                            ConstVal::Integral(ConstInt::I64(val)) => self.builder.ins().iconst(cretonne::ir::types::I64, val),
                            ConstVal::Bool(val) => {
                                self.builder.ins().bconst(cretonne::ir::types::B1, val)
                            }
                            _ => panic!("unimplemented value: {:?}", value),
                        }
                    }
                    Literal::Promoted { .. } => panic!("unimplemented Promoted Literal: {:?}", c),
                    _ => panic!("unimplemented Constant Literal {:?}", c),
                }
            }
        }
    }

    // Imported from miri and slightly modified to adapt to our monomorphize api
    fn type_layout_with_substs(&self, ty: Ty<'tcx>, substs: &Substs<'tcx>) -> &'tcx Layout {
        // TODO: Is this inefficient? Needs investigation.
        let ty = monomorphize::apply_substs(self.tcx, substs, &ty);

        self.tcx
            .infer_ctxt()
            .enter(|infcx| {
                       // TODO: Report this error properly.
                       let param_env = ty::ParamEnv::empty(Reveal::All);
                       ty.layout(self.tcx, param_env).unwrap()
                   })
    }

    #[inline]
    fn type_size(&self, ty: Ty<'tcx>) -> usize {
        let substs = Substs::empty();
        self.type_size_with_substs(ty, substs)
    }


    // Imported from miri
    #[inline]
    fn type_size_with_substs(&self, ty: Ty<'tcx>, substs: &'tcx Substs<'tcx>) -> usize {
        self.type_layout_with_substs(ty, substs)
            .size(&self.tcx.data_layout)
            .bytes() as usize
    }

    #[inline]
    fn type_layout(&self, ty: Ty<'tcx>) -> &'tcx Layout {
        let substs = Substs::empty();
        self.type_layout_with_substs(ty, substs)
    }
}

fn rust_ty_to_cretonne(t: Ty) -> Option<cretonne::ir::Type> {
    // FIXME zero-sized-types
    if t.is_nil() || t.is_never() {
        return None;
    }

    match t.sty {
        ty::TyFloat(FloatTy::F32) => Some(cretonne::ir::types::F32),
        ty::TyFloat(FloatTy::F64) => Some(cretonne::ir::types::F64),
        ty::TyInt(IntTy::I32) |
        ty::TyUint(UintTy::U32) => Some(cretonne::ir::types::I32),
        ty::TyInt(IntTy::I64) |
        ty::TyUint(UintTy::U64) => Some(cretonne::ir::types::I64),
        _ => panic!("unsupported type {}", t.sty),
    }
}

fn sanitize_symbol(s: &str) -> String {
    s.chars()
        .map(|c| match c {
                 '<' | '>' | ' ' | '(' | ')' => '_',
                 _ => c,
             })
        .collect()
}

#[derive(Debug)]
enum CretonneCallKind {
    Direct,
    Import, // Indirect // unimplemented at the moment
}

enum CretonneBlockKind {
    Default,
    Switch(cretonne::ir::Value),
}

type CretonneIndex = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct CretonneLvalue {
    index: CretonneIndex,
    offset: Option<u32>,
    extra: LvalueExtra,
}

impl CretonneLvalue {
    fn new(index: CretonneIndex, offset: Option<u32>, extra: LvalueExtra) -> Self {
        CretonneLvalue {
            index: index,
            offset: offset,
            extra: extra,
        }
    }

    fn offset(&self, offset: u32) -> Self {
        let offset = match self.offset {
            None => Some(offset),
            Some(base_offset) => Some(base_offset + offset),
        };

        Self::new(self.index, offset, self.extra)
    }
}

// The following is imported from miri as well
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LvalueExtra {
    None,
    // Length(u64),
    // TODO: Vtable(memory::AllocId),
    DowncastVariant(usize),
}

trait IntegerExt {
    fn size(self) -> Size;
}

impl IntegerExt for layout::Integer {
    fn size(self) -> Size {
        use rustc::ty::layout::Integer::*;
        match self {
            I1 | I8 => Size::from_bits(8),
            I16 => Size::from_bits(16),
            I32 => Size::from_bits(32),
            I64 => Size::from_bits(64),
            I128 => panic!("i128 is not yet supported"),
        }
    }
}
