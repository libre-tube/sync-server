pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    String::from("0x")
        + &bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("")
}
