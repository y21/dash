use std::collections::{HashMap, HashSet};

use error::Error;
use passes::bb_generation::{BBGenerationCtxt, BBGenerationQuery, BasicBlockMap, Labels};
use passes::type_infer::{TypeInferCtxt, TypeInferQuery, TypeMap, TypeStack};

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
    let Labels(labels) = passes::bb_generation::find_labels(bytecode).unwrap();

    let mut bcx = BBGenerationCtxt {
        bytecode,
        labels,
        bbs: HashMap::new(),
        query,
    };
    bcx.find_bbs();
    bcx.resolve_edges();

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
