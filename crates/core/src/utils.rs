use provenance_log::{Error as PlogError, Log};

use crate::Error;

/// Prints the Plog

fn print_plog(plog: &Log) -> Result<(), Error> {
    // get the verifying iterator
    let mut vi = plog.verify();

    // process the first entry and get the results
    let (_, _, mut kvp) = vi.next().ok_or::<Error>(PlogError::NoFirstEntry.into())??;
    let vlad_key_value = kvp
        .get("/vlad/key")
        .ok_or::<Error>(PlogError::NoVladKey.into())?;
    let vlad_key: Multikey =
        get_from_wacc_value(&vlad_key_value).ok_or::<Error>(PlogError::InvalidWaccValue.into())?;

    for ret in vi {
        match ret {
            Ok((_, _, ref pairs)) => kvp = pairs.clone(),
            Err(e) => debug!("verify failed: {}", e.to_string()),
        }
    }

    let vl: Vec<String> = format!(
        "(vlad) {}",
        EncodedVlad::new(Base::Base32Z, plog.vlad.clone())
    )
    .chars()
    .collect::<Vec<_>>()
    .chunks(83)
    .map(|chunk| chunk.iter().collect())
    .collect();
    for l in &vl {
        println!("│  {}", l);
    }

    if plog.vlad.verify(&vlad_key).is_ok() {
        let fingerprint = vlad_key.fingerprint_view()?.fingerprint(Codec::Blake3)?;
        let ef = EncodedMultihash::new(Base::Base32Z, fingerprint);
        println!(
            "│   ╰─ ☑ verified '/vlad/key' -> ({}) {}",
            vlad_key.codec(),
            ef
        );
    } else {
        println!("│   ╰─ ☒ failed to verify");
    }
    println!("├─ entries {}", plog.entries.len());
    println!("╰─ kvp");
    for (i, (k, v)) in kvp.iter().enumerate() {
        if i < kvp.len() - 1 {
            print!("    ├─ '{}' -> ", k);
        } else {
            print!("    ╰─ '{}' -> ", k);
        }
        if let Some(codec) = get_codec_from_plog_value(v) {
            match codec {
                Codec::Multikey => {
                    let v = kvp
                        .get(k.as_str())
                        .ok_or::<Error>(PlogError::NoKeyPath.into())?;
                    let key: Multikey = get_from_wacc_value(&v)
                        .ok_or::<Error>(PlogError::InvalidWaccValue.into())?;
                    let fingerprint = key.fingerprint_view()?.fingerprint(Codec::Blake3)?;
                    let ef = EncodedMultihash::new(Base::Base32Z, fingerprint);
                    println!("({} key) {}", key.codec(), ef);
                }
                Codec::Vlad => {
                    let v = kvp
                        .get(k.as_str())
                        .ok_or::<Error>(PlogError::NoKeyPath.into())?;
                    let vlad: Vlad = get_from_wacc_value(&v)
                        .ok_or::<Error>(PlogError::InvalidWaccValue.into())?;
                    println!("(vlad) {}", EncodedVlad::new(Base::Base32Z, vlad.clone()));
                }
                Codec::ProvenanceLogScript => {
                    let v = kvp
                        .get(k.as_str())
                        .ok_or::<Error>(PlogError::NoKeyPath.into())?;
                    let script: Script = get_from_wacc_value(&v)
                        .ok_or::<Error>(PlogError::InvalidWaccValue.into())?;
                    println!("(script) {} bytes", script.as_ref().len());
                }
                Codec::Cidv1 | Codec::Cidv2 | Codec::Cidv3 => {
                    let v = kvp
                        .get(k.as_str())
                        .ok_or::<Error>(PlogError::NoKeyPath.into())?;
                    let cid: Cid = get_from_wacc_value(&v)
                        .ok_or::<Error>(PlogError::InvalidWaccValue.into())?;
                    println!(
                        "({}) {}",
                        cid.codec(),
                        EncodedCid::new(Base::Base32Z, cid.clone())
                    );
                }
                _ => println!("{}", codec),
            }
        } else {
            match v {
                provenance_log::Value::Data(v) => println!("data of length {}", v.len()),
                provenance_log::Value::Str(s) => println!("'{}'", s),
                _ => println!("Nil"),
            }
        }
    }
    /*
    let kvp_lines = kvp.to_string().lines().map(|s| s.to_string()).collect::<Vec<_>>();
    for i in 0..kvp_lines.len() {
        if i < kvp_lines.len() - 1 {
            println!("    ├─ {}", kvp_lines[i]);
        } else {
            println!("    ╰─ {}", kvp_lines[i]);
        }
    }
    */
    Ok(())
}
