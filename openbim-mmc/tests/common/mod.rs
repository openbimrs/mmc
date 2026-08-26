#![allow(dead_code)]

use std::io::{Cursor, Write};

use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

pub const MMC_NS: &str = "http://www.buildingsmart.org/multi-model/MMContainer/2.0.0";
pub const LINK_NS: &str = "http://www.buildingsmart.org/multi-model/LinkModel/2.0.0";
pub const UUID: &str = "4d69a342-31b6-4e80-9d05-83a28754c84d";

pub fn valid_multimodel(link_path: &str, model_path: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mmc:MultiModel xmlns:mmc="{MMC_NS}" uuid="{UUID}" formatVersion="2.0.0" mmDomain="urn:din:18290:test">
  <mmc:MetaData><mmc:Meta key="projectName" value="Bridge &amp; Road"/></mmc:MetaData>
  <mmc:ApplicationModels>
    <mmc:ApplicationModel id="model-ifc" modelType="IFC">
      <mmc:ModelData id="ifc-spf" formatType="IFC" formatVersion="IFC4">
        <mmc:DataRessource id="ifc-file" location="{model_path}"/>
      </mmc:ModelData>
    </mmc:ApplicationModel>
    <mmc:ApplicationModel id="model-gaeb" modelType="GAEB">
      <mmc:ModelData id="gaeb-xml" formatType="GAEB DA XML" formatVersion="3.3">
        <mmc:DataRessource id="gaeb-file" location="https://example.test/bill.xml"/>
      </mmc:ModelData>
    </mmc:ApplicationModel>
  </mmc:ApplicationModels>
  <mmc:LinkModels>
    <mmc:LinkModel location="{link_path}">
      <mmc:LinkedModel>model-ifc</mmc:LinkedModel>
      <mmc:LinkedModel>model-gaeb</mmc:LinkedModel>
    </mmc:LinkModel>
  </mmc:LinkModels>
</mmc:MultiModel>"#,
    )
    .into_bytes()
}

pub fn valid_link_model() -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<l:LinkModel xmlns:l="{LINK_NS}" formatVersion="2.0.0">
  <l:MetaData><l:Meta k="role" v="quantity"/></l:MetaData>
  <l:Link>
    <l:Relatum m="model-ifc" id="2AbC" f="ifc-spf" r="ifc-file"/>
    <l:Relatum m="model-gaeb" id="item-1"/>
  </l:Link>
</l:LinkModel>"#,
    )
    .into_bytes()
}

pub fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o100644);
        for (name, bytes) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    output.into_inner()
}

pub fn zip_with_mode(name: &str, bytes: &[u8], mode: u32) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(mode);
        writer.start_file(name, options).unwrap();
        writer.write_all(bytes).unwrap();
        writer.finish().unwrap();
    }
    let mut bytes = output.into_inner();
    let central = bytes
        .windows(4)
        .position(|window| window == b"PK\x01\x02")
        .expect("central directory");
    bytes[central + 5] = 3;
    bytes[central + 38..central + 42].copy_from_slice(&(mode << 16).to_le_bytes());
    bytes
}

pub fn zip_with_exact_duplicate_root(index: &[u8]) -> Vec<u8> {
    let mut bytes = zip(&[("MultiModel.xml", index), ("MultiModel.xMl", b"other")]);
    let old = b"MultiModel.xMl";
    let new = b"MultiModel.xml";
    let mut replacements = 0;
    for offset in 0..=bytes.len() - old.len() {
        if &bytes[offset..offset + old.len()] == old {
            bytes[offset..offset + old.len()].copy_from_slice(new);
            replacements += 1;
        }
    }
    assert_eq!(
        replacements, 2,
        "local and central ZIP names must be replaced"
    );
    bytes
}

pub fn valid_archive() -> Vec<u8> {
    let index = valid_multimodel("links/elements.xml", "models/model.ifc");
    let links = valid_link_model();
    zip(&[
        ("MultiModel.xml", index.as_slice()),
        ("models/model.ifc", b"ISO-10303-21;ENDSEC;END-ISO-10303-21;"),
        ("links/elements.xml", links.as_slice()),
    ])
}
