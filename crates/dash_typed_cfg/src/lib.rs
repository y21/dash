use std::collections::HashMap;
use std::collections::HashSet;

use error::Error;
use passes::bb_generation::BBGenerationCtxt;
use passes::bb_generation::BBGenerationQuery;
use passes::bb_generation::BasicBlockMap;
use passes::bb_generation::Labels;
use passes::type_infer::TypeInferCtxt;
use passes::type_infer::TypeInferQuery;
use passes::type_infer::TypeMap;
use passes::type_infer::TypeStack;

pub mod error;
pub mod passes;
pub mod util;

pub trait TypedCfgQuery: TypeInferQuery + BBGenerationQuery {}

#[derive(Debug)]
pub struct TypedCfg {
    pub ty_map: TypeMap,
    pub bb_map: BasicBlockMap,
}

pub fn lower<Q: TypedCfgQuery>(bytecode: &[u8], query: &mut Q) -> Result<TypedCfg, Error> {
    let Labels(labels) = passes::bb_generation::find_labels(bytecode)?;

    let mut bcx = BBGenerationCtxt {
        bytecode,
        labels,
        bbs: HashMap::new(),
        query,
    };
    bcx.find_bbs();
    bcx.resolve_edges()?;

    let mut tycx = TypeInferCtxt {
        bbs: bcx.bbs,
        bytecode,
        local_tys: HashMap::new(),
        query,
        visited: HashSet::new(),
    };
    tycx.resolve_types(TypeStack::default(), 0)?;

    Ok(TypedCfg {
        bb_map: tycx.bbs,
        ty_map: tycx.local_tys,
    })
}
