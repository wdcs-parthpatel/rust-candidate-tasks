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
use std::collections::HashMap;
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
struct TestData {
    id: u32,
    name: String,
    flag: bool,
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
    let data = TestData {
        id: 12345,
        name: "SerializationTest".to_string(),
        flag: true,
    };

    let mut result = HashMap::new();

    let json_ser_time = benchmark(|| serde_json::to_vec(&data).unwrap());
    let json = serde_json::to_vec(&data).unwrap();
    let json_de_time = benchmark(|| serde_json::from_slice::<TestData>(&json).unwrap());
    result.insert("serde_json", (json.len(), json_ser_time, json_de_time));

    let bin_ser_time = benchmark(|| bincode_encode(&data, standard()).unwrap());
    let bin = bincode_encode(&data, standard()).unwrap();
    let bin_de_time = benchmark(|| bincode_decode::<TestData, _>(&bin, standard()).unwrap());
    result.insert("bincode", (bin.len(), bin_ser_time, bin_de_time));

    let bcs_ser_time = benchmark(|| bcs::to_bytes(&data).unwrap());
    let bcs = bcs::to_bytes(&data).unwrap();
    let bcs_de_time = benchmark(|| bcs::from_bytes::<TestData>(&bcs).unwrap());
    result.insert("bcs", (bcs.len(), bcs_ser_time, bcs_de_time));

    let borsh_ser_time = benchmark(|| borsh::to_vec(&data).unwrap());
    let borsh = borsh::to_vec(&data).unwrap();
    let borsh_de_time = benchmark(|| borsh::from_slice::<TestData>(&borsh).unwrap());
    result.insert("borsh", (borsh.len(), borsh_ser_time, borsh_de_time));

    let mut proto = test::ProtoData::new();
    proto.id = data.id;
    proto.name = data.name.clone();
    proto.flag = data.flag;

    let proto_ser_time = benchmark(|| proto.write_to_bytes().unwrap());
    let proto_data = proto.write_to_bytes().unwrap();
    let proto_de_time = benchmark(|| test::ProtoData::parse_from_bytes(&proto_data).unwrap());
    result.insert(
        "protobuf",
        (proto_data.len(), proto_ser_time, proto_de_time),
    );

    let rmp_ser_time = benchmark(|| rmp_encode::to_vec_named(&data).unwrap());
    let rmp = rmp_encode::to_vec_named(&data).unwrap();
    let rmp_get_time = benchmark(|| rmp_decode::from_slice::<TestData>(&rmp).unwrap());
    result.insert("rmp", (rmp.len(), rmp_ser_time, rmp_get_time));

    let rkyv_ser_time = benchmark(|| to_bytes::<Error>(&data).unwrap());
    let rkyv_bytes = to_bytes::<Error>(&data).unwrap();
    let rkyv_de_time = benchmark(|| from_bytes::<TestData, Error>(&rkyv_bytes).unwrap());
    result.insert("rkyv", (rkyv_bytes.len(), rkyv_ser_time, rkyv_de_time));

    println!("Format     | Size | Serialize (ms) | Deserialize (ms)");
    println!("-----------|------|----------------|------------------");
    for (format, (size, ser_time, de_time)) in result {
        println!(
            "{:<10} | {:>4} | {:>14.3} | {:>16.3}",
            format,
            size,
            ser_time.as_secs_f64() * 1000.0,
            de_time.as_secs_f64() * 1000.0
        );
    }
}
