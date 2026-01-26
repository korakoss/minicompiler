use crate::shared::tables::*;
use std::collections::HashMap;

#[test]
fn test_rank_primitive() {
    let mut table = GenericTypetable::new(HashMap::new());
    let pr_rank = table.get_genericity_rank(&ConcreteType::Prim(PrimType::Bool));
    assert!(pr_rank == 0);
}


#[test]
fn test_rank_reference() {
    let mut table = GenericTypetable::new(HashMap::new());
    let ref_type = ConcreteType::Reference(Box::new(ConcreteType::Prim(PrimType::Integer)));
    let pr_rank = table.get_genericity_rank(&ref_type);
    assert!(pr_rank == 1);
}


#[test]
fn test_rank_struct() {
    let structdef = GenericTypeDef {
        type_params: vec!["T".to_string()],
        defn: GenericShape::Struct { 
            fields: BTreeMap::from([("a".to_string(), GenericType::TypeVar("T".to_string()))]), 
        }
    };
    let mut table = GenericTypetable::new(
        HashMap::from([(NewtypeId("Gen".to_string()), structdef)])
    );
    let ctyp = ConcreteType::NewType(NewtypeId("Gen".to_string()), vec![ConcreteType::Prim(PrimType::Integer)]);
    assert!(table.get_genericity_rank(&ctyp) == 1);
}
