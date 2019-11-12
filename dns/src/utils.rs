// qname 转换为可读的字符串
pub fn qname_to_domain(qname: &[u8]) -> String {
    let mut labels = Vec::new();

    let mut index = 0;
    while index < qname.len() {
        let label_len: usize = qname[index] as usize;

        if label_len == 0 {
            break;
        }

        let label_begin_index = index + 1;
        let next_index = label_begin_index + label_len;

        let label_bytes = &qname[label_begin_index..next_index];

        let label = String::from_utf8_lossy(label_bytes).to_string();
        labels.push(label);

        index = next_index
    }

    labels.join(".")
}

// 把域名从转换为 C 风格的字符串
pub fn domain_to_qname(domain: &str) -> Vec<u8> {
    let mut qname: Vec<u8> = Vec::new();

    let labels: Vec<&str> = domain.split('.').collect();
    for label in labels {
        qname.push(label.len() as u8);

        for c in label.as_bytes() {
            qname.push(*c);
        }
    }

    qname.push(0);

    qname
}
