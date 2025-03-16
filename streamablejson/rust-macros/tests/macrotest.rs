use streamablejson_macros::streamablejson;

#[test]
fn test_object() {
    let _json = streamablejson!({
        "name": "John Doe",
        "age": 30,
        "city": "New York"
    });
}

#[test]
fn test_array() {
    let _json = streamablejson!([abc, def]);

}