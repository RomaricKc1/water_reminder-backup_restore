use csv::{ReaderBuilder, StringRecord};

pub fn read_csv_to_vec(path: String) -> Result<Vec<String>, String> {
    let mut rdr = match ReaderBuilder::new().has_headers(false).from_path(path) {
        Err(e) => {
            return Err(format!("Error occured: {e}"));
        }
        Ok(r) => r,
    };

    let mut records: Vec<StringRecord> = Vec::new();

    for result in rdr.records() {
        let record = match result {
            Err(e) => {
                return Err(format!("Error occured: {e}"));
            }
            Ok(r) => r,
        };
        records.push(record);
        // s_record = Some(record);
    }
    let res = records
        .iter()
        .map(|r| r.iter().collect::<Vec<&str>>().join(","))
        .collect::<Vec<String>>();

    Ok(res)
}

pub fn read_csv_to_str(path: String) -> Result<String, String> {
    let mut rdr = match ReaderBuilder::new().has_headers(false).from_path(path) {
        Err(e) => {
            return Err(format!("Error occured: {e}"));
        }
        Ok(r) => r,
    };

    // let mut s_record: Option<StringRecord> = None;
    let mut records: Vec<StringRecord> = Vec::new();

    for result in rdr.records() {
        let record = match result {
            Err(e) => {
                return Err(format!("Error occured: {e}"));
            }
            Ok(r) => r,
        };
        records.push(record);
        // s_record = Some(record);
    }
    let res = records
        .iter()
        .map(|r| r.iter().collect::<Vec<&str>>().join(","))
        .collect::<Vec<String>>()
        .join("\n");

    // if let Some(record) = s_record {
    //     let any = record.iter().map(|r| r.iter);
    // }

    Ok(res)
}

pub fn str_to_csv(data: String, save_path: String) {
    match std::fs::write(save_path.clone(), data) {
        Err(e) => {
            println!("Error occured: {e}");
        }
        Ok(_) => {
            println!("wrote to file {save_path}.csv");
        }
    };
}
