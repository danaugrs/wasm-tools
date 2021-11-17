use crate::{
    module::{PrimitiveTypeInfo, TypeInfo},
    mutators::peephole::eggsy::Lang,
};
use egg::{Analysis, EGraph, Id};

/// Analysis implementation for our defined language
/// It will maintain the information regarding to map eterm to wasm and back: the DFG, the symbols
/// and the mapping between the equivalence classes and the stack entry in the DFG of the Wasm basic block
pub struct PeepholeMutationAnalysis {
    /// Module information for globals
    global_types: Vec<PrimitiveTypeInfo>,
    /// Module information for function locals
    locals: Vec<PrimitiveTypeInfo>,

    /// Information from the ModuleInfo
    /// types for functions
    types_map: Vec<TypeInfo>,
    /// function idx to type idx
    function_map: Vec<u32>,
}

impl PeepholeMutationAnalysis {
    /// Returns a new analysis from the given DFG
    pub fn new(
        global_types: Vec<PrimitiveTypeInfo>,
        locals: Vec<PrimitiveTypeInfo>,
        types_map: Vec<TypeInfo>,
        function_map: Vec<u32>,
    ) -> Self {
        PeepholeMutationAnalysis {
            global_types,
            locals,
            types_map,
            function_map,
        }
    }

    /// Gets returning type of node
    pub fn get_returning_tpe(&self, l: &Lang) -> crate::Result<PrimitiveTypeInfo> {
        match l {
            Lang::I32Add(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Add(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Sub(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Sub(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Mul(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Mul(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32And(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64And(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Or(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Or(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Xor(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Xor(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Shl(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Shl(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32ShrU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64ShrU(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32DivU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64DivU(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32DivS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64DivS(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32ShrS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64ShrS(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32RotR(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64RotR(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32RotL(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64RotL(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32RemS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64RemS(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32RemU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64RemU(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Eqz(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Eqz(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32Eq(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Eq(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32Ne(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Ne(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32LtS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64LtS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32LtU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64LtU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32GtS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64GtS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32GtU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64GtU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32LeS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64LeS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32LeU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64LeU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32GeS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64GeS(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I32GeU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64GeU(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::LocalTee(idx, _) => Ok(self.locals[*idx as usize].clone()),
            Lang::Wrap(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::Call(idx, _) => {
                let typeinfo = self.get_functype_idx(*idx);

                match typeinfo {
                    TypeInfo::Func(ty) => {
                        if ty.returns.is_empty() {
                            return Ok(PrimitiveTypeInfo::Empty);
                        }

                        if ty.returns.len() > 1 {
                            return Err(crate::Error::NoMutationsApplicable);
                        }

                        Ok(ty.returns[0].clone())
                    }
                    _ => unreachable!("Invalid function type {:?}", typeinfo),
                }
            }
            Lang::I32Popcnt(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Popcnt(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::Drop(_) => Ok(PrimitiveTypeInfo::Empty),
            Lang::I32Load { .. } => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Load { .. } => Ok(PrimitiveTypeInfo::I64),
            Lang::RandI32 => Ok(PrimitiveTypeInfo::I32),
            Lang::RandI64 => Ok(PrimitiveTypeInfo::I64),
            Lang::Undef => Ok(PrimitiveTypeInfo::Empty),
            Lang::UnfoldI32(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::UnfoldI64(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Extend8S(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Extend8S(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I32Extend16S(_) => Ok(PrimitiveTypeInfo::I32),
            Lang::I64Extend16S(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I64Extend32S(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I64ExtendI32S(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::I64ExtendI32U(_) => Ok(PrimitiveTypeInfo::I64),
            Lang::LocalSet(_, _) => Ok(PrimitiveTypeInfo::Empty),
            Lang::GlobalSet(_, _) => Ok(PrimitiveTypeInfo::Empty),
            Lang::LocalGet(idx) => Ok(self.locals[*idx as usize].clone()),
            Lang::GlobalGet(v) => Ok(self.global_types[*v as usize].clone()),
            Lang::I32Store { .. } => Ok(PrimitiveTypeInfo::Empty),
            Lang::I64Store { .. } => Ok(PrimitiveTypeInfo::Empty),
            // Lang::Select(_) => todo!(),
            Lang::F32(_) => Ok(PrimitiveTypeInfo::F32),
            Lang::F64(_) => Ok(PrimitiveTypeInfo::I64),
        }
    }

    /// Returns the function type based on the index of the function type
    /// `types[functions[idx]]`
    pub fn get_functype_idx(&self, idx: usize) -> &TypeInfo {
        let functpeindex = self.function_map[idx] as usize;
        &self.types_map[functpeindex]
    }
}

#[derive(Debug, Clone)]
pub struct ClassData {
    /// Type 't' of the operator
    /// 't'.op
    pub tpe: PrimitiveTypeInfo,
}

impl PartialEq for ClassData {
    fn eq(&self, other: &Self) -> bool {
        self.tpe == other.tpe
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Analysis<Lang> for PeepholeMutationAnalysis {
    type Data = Option<ClassData>;

    fn make(egraph: &EGraph<Lang, Self>, l: &Lang) -> Self::Data {
        // We build the nodes collection in the same order the egraph is built
        Some(ClassData {
            // This type information is used only when the rewriting rules are being applied ot the egraph
            // Thats why we need the original expression in the analysis beforehand :)
            // Beyond that the random extracted expression needs to be pass to the `get_returning_tpe` method
            tpe: egraph.analysis.get_returning_tpe(l).expect("Missing type"),
        })
    }

    fn merge(&self, to: &mut Self::Data, from: Self::Data) -> bool {
        egg::merge_if_different(to, to.clone().or(from))
    }

    fn modify(_: &mut EGraph<Lang, Self>, _: Id) {}
}
