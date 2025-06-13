mod generated;

use bincode::config::standard;
use bincode::serde::{decode_from_slice as bincode_decode, encode_to_vec as bincode_encode};
use borsh::{BorshDeserialize, BorshSerialize};
use generated::test;
use protobuf::Message;
use rkyv::{
    Archive, Deserialize as rkyv_deserialize, Serialize as rkyv_serialize, from_bytes,
    rancor::Error, to_bytes,
};
use rmp_serde::{decode as rmp_decode, encode as rmp_encode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::time::{Duration, Instant};

const ITERATION_COUNT: usize = 10000;

#[derive(
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Archive,
    rkyv_deserialize,
    rkyv_serialize,
    Debug,
    PartialEq,
)]

struct Person {
    first_name: String,
    middle_name: String,
    last_name: String,
    birth_date: String,
    age: u8,
    is_adult: bool,
}

fn benchmark<T, F>(mut func: F) -> Duration
where
    F: FnMut() -> T,
{
    let now = Instant::now();
    for _ in 0..ITERATION_COUNT {
        func();
    }
    now.elapsed()
}

fn main() {
    let data = Person {
        first_name: "Parth".to_string(),
        middle_name: "Dipakbhai".to_string(),
        last_name: "Patel".to_string(),
        birth_date: "06 Jan 1991".to_string(),
        age: 34,
        is_adult: true,
    };

    let mut result = HashMap::new();

    let json_ser_time = benchmark(|| serde_json::to_vec(&data).unwrap());
    let json = serde_json::to_vec(&data).unwrap();
    let json_de_time = benchmark(|| serde_json::from_slice::<Person>(&json).unwrap());
    result.insert("serde_json", (json.len(), json_ser_time, json_de_time));

    let bin_ser_time = benchmark(|| bincode_encode(&data, standard()).unwrap());
    let bin = bincode_encode(&data, standard()).unwrap();
    let bin_de_time = benchmark(|| bincode_decode::<Person, _>(&bin, standard()).unwrap());
    result.insert("bincode", (bin.len(), bin_ser_time, bin_de_time));

    let bcs_ser_time = benchmark(|| bcs::to_bytes(&data).unwrap());
    let bcs = bcs::to_bytes(&data).unwrap();
    let bcs_de_time = benchmark(|| bcs::from_bytes::<Person>(&bcs).unwrap());
    result.insert("bcs", (bcs.len(), bcs_ser_time, bcs_de_time));

    let borsh_ser_time = benchmark(|| borsh::to_vec(&data).unwrap());
    let borsh = borsh::to_vec(&data).unwrap();
    let borsh_de_time = benchmark(|| borsh::from_slice::<Person>(&borsh).unwrap());
    result.insert("borsh", (borsh.len(), borsh_ser_time, borsh_de_time));

    let mut proto = test::ProtoData::new();
    proto.first_name = data.first_name.clone();
    proto.middle_name = data.middle_name.clone();
    proto.last_name = data.last_name.clone();
    proto.birth_date = data.birth_date.clone();
    proto.age = u32::from(data.age);
    proto.is_adult = data.is_adult;

    let proto_ser_time = benchmark(|| proto.write_to_bytes().unwrap());
    let proto_data = proto.write_to_bytes().unwrap();
    let proto_de_time = benchmark(|| test::ProtoData::parse_from_bytes(&proto_data).unwrap());
    result.insert(
        "protobuf",
        (proto_data.len(), proto_ser_time, proto_de_time),
    );

    let rmp_ser_time = benchmark(|| rmp_encode::to_vec_named(&data).unwrap());
    let rmp = rmp_encode::to_vec_named(&data).unwrap();
    let rmp_get_time = benchmark(|| rmp_decode::from_slice::<Person>(&rmp).unwrap());
    result.insert("rmp", (rmp.len(), rmp_ser_time, rmp_get_time));

    let rkyv_ser_time = benchmark(|| to_bytes::<Error>(&data).unwrap());
    let rkyv_bytes = to_bytes::<Error>(&data).unwrap();
    let rkyv_de_time = benchmark(|| from_bytes::<Person, Error>(&rkyv_bytes).unwrap());
    result.insert("rkyv", (rkyv_bytes.len(), rkyv_ser_time, rkyv_de_time));

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("benchmark_result.txt")
        .expect("Opening file failed");

    let format = "\nFormat     | Size | Serialize (ms) | Deserialize (ms)".to_string();
    writeln!(file, "{}", format).expect("Writing file failed");
    writeln!(
        file,
        "-----------|------|----------------|------------------"
    )
    .expect("Writing file failed");

    println!("{}", format);
    println!("-----------|------|----------------|------------------");

    let btree_map: BTreeMap<&str, (usize, Duration, Duration)> = result.into_iter().collect();
    for (format, (size, ser_time, de_time)) in btree_map {
        let output = format!(
            "{:<10} | {:>4} | {:>14.3} | {:>16.3}",
            format,
            size,
            ser_time.as_secs_f64() * 1000.0,
            de_time.as_secs_f64() * 1000.0
        );
        writeln!(file, "{}", output).expect("Writing file failed");
        println!("{}", output);
    }
}
