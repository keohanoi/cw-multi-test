use cw_multi_test::App;

#[test]
fn test_ffi() {
    let app = App::default();
    let address = "0x10505818AFDB5fA60862e1D771a84E8164Dd9D49";
    let args = &[
        "npm".to_string(),
        "--prefix".to_string(),
        "tests/test_app_builder/ts".to_string(),
        "--silent".to_string(),
        "run".to_string(),
        "test".to_string(),
    ];
    let output = app.ffi(args).unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), address)
}
