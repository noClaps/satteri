use tryckeri_plugin_api::*;

/// 1. DataMap: set and get string value
#[test]
fn data_map_set_and_get_string() {
    let mut map = DataMap::new();
    map.set(1, "title", DataValue::String("Hello World".to_string()));
    let val = map.get(1, "title").expect("value should be present");
    assert_eq!(val.as_str().unwrap(), "Hello World");
}

/// 2. DataMap: entries_for_node filters correctly
#[test]
fn data_map_entries_for_node() {
    let mut map = DataMap::new();
    map.set(1, "a", DataValue::Int(1));
    map.set(1, "b", DataValue::Bool(true));
    map.set(2, "a", DataValue::String("other".to_string()));

    let entries: Vec<_> = map.entries_for_node(1).collect();
    assert_eq!(entries.len(), 2, "node 1 should have 2 entries");

    let keys: Vec<&str> = entries.iter().map(|(k, _)| *k).collect();
    assert!(keys.contains(&"a"), "should have key 'a'");
    assert!(keys.contains(&"b"), "should have key 'b'");

    let entries2: Vec<_> = map.entries_for_node(2).collect();
    assert_eq!(entries2.len(), 1, "node 2 should have 1 entry");
}

/// 3. TypedDataMap: set and get typed value (use a custom struct)
#[test]
fn typed_data_map_set_and_get() {
    #[derive(Debug, PartialEq)]
    struct MyData { value: i32, label: String }

    let mut map = TypedDataMap::new();
    map.set(42u32, MyData { value: 99, label: "test".to_string() });

    let retrieved = map.get::<MyData>(42).expect("should retrieve typed data");
    assert_eq!(retrieved.value, 99);
    assert_eq!(retrieved.label, "test");
}

/// 4. TypedDataMap: different types for same node_id don't conflict
#[test]
fn typed_data_map_different_types_same_node() {
    #[derive(Debug, PartialEq)]
    struct TypeA(i32);
    #[derive(Debug, PartialEq)]
    struct TypeB(String);

    let mut map = TypedDataMap::new();
    map.set(1u32, TypeA(100));
    map.set(1u32, TypeB("hello".to_string()));

    let a = map.get::<TypeA>(1).expect("TypeA should exist");
    let b = map.get::<TypeB>(1).expect("TypeB should exist");
    assert_eq!(a.0, 100);
    assert_eq!(b.0, "hello");
}

/// 5. TypedDataMap: has() returns false before set, true after
#[test]
fn typed_data_map_has_before_and_after_set() {
    #[derive(Debug)]
    struct Marker;

    let mut map = TypedDataMap::new();
    assert!(!map.has::<Marker>(7), "should not have data before set");
    map.set(7u32, Marker);
    assert!(map.has::<Marker>(7), "should have data after set");
}

/// Extra: DataMap remove works
#[test]
fn data_map_remove() {
    let mut map = DataMap::new();
    map.set(1, "key", DataValue::Bool(true));
    assert!(map.has(1, "key"));
    map.remove(1, "key");
    assert!(!map.has(1, "key"));
}

/// Extra: DataMap len and is_empty
#[test]
fn data_map_len_and_is_empty() {
    let mut map = DataMap::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    map.set(1, "x", DataValue::Null);
    assert!(!map.is_empty());
    assert_eq!(map.len(), 1);
}

/// Extra: TypedDataMap remove
#[test]
fn typed_data_map_remove() {
    #[derive(Debug)]
    struct Tag(u32);

    let mut map = TypedDataMap::new();
    map.set(5u32, Tag(99));
    assert!(map.has::<Tag>(5));
    map.remove::<Tag>(5);
    assert!(!map.has::<Tag>(5));
}

/// Extra: DataValue helper methods
#[test]
fn data_value_helpers() {
    let s = DataValue::String("hello".to_string());
    assert_eq!(s.as_str(), Some("hello"));
    assert_eq!(s.as_bool(), None);
    assert_eq!(s.as_int(), None);

    let b = DataValue::Bool(true);
    assert_eq!(b.as_bool(), Some(true));
    assert_eq!(b.as_str(), None);

    let i = DataValue::Int(42);
    assert_eq!(i.as_int(), Some(42));
    assert_eq!(i.as_str(), None);
}
