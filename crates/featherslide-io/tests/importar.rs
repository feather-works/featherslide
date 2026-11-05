//! Ler `.pptx` e `.odp`: arquivos mínimos montados aqui, com o que os de verdade têm de
//! traiçoeiro — a caixa de título sem posição (herda do layout), a cor do tema passando pelo
//! mapa do mestre, o tamanho da letra vindo dos estilos do mestre, grupos, listas.

use std::io::{Cursor, Write};

use featherslide_core::modelo::{Papel, Proporcao, Tipo};
use featherslide_io::{odp, pptx};
use zip::write::SimpleFileOptions;

fn zip(partes: &[(&str, &str)]) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (nome, conteudo) in partes {
        z.start_file(*nome, SimpleFileOptions::default()).unwrap();
        z.write_all(conteudo.as_bytes()).unwrap();
    }
    z.finish().unwrap().into_inner()
}

const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;

fn rels(itens: &[(&str, &str)]) -> String {
    let r: String = itens.iter().map(|(id, alvo)| format!(r#"<Relationship Id="{id}" Type="x" Target="{alvo}"/>"#)).collect();
    format!(r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{r}</Relationships>"#)
}

fn pptx_minimo() -> Vec<u8> {
    let pres = format!(r#"<p:presentation {NS}><p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:sldSz cx="9144000" cy="5143500"/></p:presentation>"#);
    let tema = r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:themeElements><a:clrScheme name="x"><a:dk1><a:srgbClr val="111111"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:accent1><a:srgbClr val="FF0000"/></a:accent1></a:clrScheme><a:fontScheme name="f"><a:majorFont><a:latin typeface="Georgia"/></a:majorFont><a:minorFont><a:latin typeface="Verdana"/></a:minorFont></a:fontScheme></a:themeElements></a:theme>"#;
    let mestre = format!(r#"<p:sldMaster {NS}><p:cSld><p:bg><p:bgPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></p:bgPr></p:bg><p:spTree/></p:cSld><p:clrMap bg1="lt1" tx1="dk1"/><p:txStyles><p:titleStyle><a:lvl1pPr algn="ctr"><a:defRPr sz="4400" b="1"><a:latin typeface="+mj-lt"/></a:defRPr></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:buChar char="•"/><a:defRPr sz="2400"/></a:lvl1pPr></p:bodyStyle><p:otherStyle/></p:txStyles></p:sldMaster>"#);
    let layout = format!(r#"<p:sldLayout {NS} type="obj"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="2" name="t"/><p:cNvSpPr/><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="457200" y="228600"/><a:ext cx="8229600" cy="857250"/></a:xfrm></p:spPr></p:sp></p:spTree></p:cSld></p:sldLayout>"#);
    let slide = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree>
<p:sp><p:nvSpPr><p:cNvPr id="2" name="t"/><p:cNvSpPr/><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:p><a:r><a:rPr lang="pt-BR"/><a:t>Título herdado</a:t></a:r></a:p></p:txBody></p:sp>
<p:sp><p:nvSpPr><p:cNvPr id="3" name="c"/><p:cNvSpPr/><p:nvPr><p:ph idx="1"/></p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="457200" y="1200000"/><a:ext cx="8229600" cy="3000000"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:p><a:r><a:t>primeiro</a:t></a:r></a:p><a:p><a:r><a:t>segundo</a:t></a:r></a:p></p:txBody></p:sp>
<p:grpSp><p:grpSpPr><a:xfrm><a:off x="1000000" y="1000000"/><a:ext cx="2000000" cy="1000000"/><a:chOff x="0" y="0"/><a:chExt cx="4000000" cy="2000000"/></a:xfrm></p:grpSpPr>
<p:sp><p:nvSpPr><p:cNvPr id="4" name="e"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="4000000" cy="2000000"/></a:xfrm><a:prstGeom prst="ellipse"/><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill></p:spPr></p:sp>
</p:grpSp>
</p:spTree></p:cSld></p:sld>"#
    );
    zip(&[
        ("ppt/presentation.xml", &pres),
        ("ppt/_rels/presentation.xml.rels", &rels(&[("rId1", "slideMasters/slideMaster1.xml"), ("rId2", "slides/slide1.xml")])),
        ("ppt/slides/slide1.xml", &slide),
        ("ppt/slides/_rels/slide1.xml.rels", &rels(&[("rId1", "../slideLayouts/slideLayout1.xml")])),
        ("ppt/slideLayouts/slideLayout1.xml", &layout),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", &rels(&[("rId1", "../slideMasters/slideMaster1.xml")])),
        ("ppt/slideMasters/slideMaster1.xml", &mestre),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels", &rels(&[("rId1", "../theme/theme1.xml")])),
        ("ppt/theme/theme1.xml", tema),
    ])
}

#[test]
fn pptx_herda_posicao_cor_fonte_e_lista() {
    let ap = pptx::ler(&pptx_minimo()).unwrap();
    assert_eq!(ap.proporcao, Proporcao::Larga);
    let s = &ap.slides[0];
    assert_eq!(s.fundo, "#ff0000", "fundo do mestre, cor do tema");
    assert_eq!(s.layout, "titulo_conteudo");
    let titulo = s.elementos.iter().find(|e| e.papel == Papel::Titulo).expect("título");
    assert_eq!(titulo.texto, "Título herdado");
    // 457200 EMU = 36 pt no slide de 720 pt; o FeatherSlide tem 960 pt: × 4/3. A margem interna
    // padrão (7,2 pt) também cresce × 4/3 e fica guardada na caixa.
    assert!((titulo.x - 48.0).abs() < 0.1, "x herdado do layout: {}", titulo.x);
    assert!((titulo.margens()[0] - 9.6).abs() < 0.01, "margem: {:?}", titulo.margem);
    assert!((titulo.estilo.tamanho - 44.0 * 4.0 / 3.0).abs() < 0.2, "tamanho do mestre: {}", titulo.estilo.tamanho);
    assert!(titulo.estilo.negrito);
    assert_eq!(titulo.estilo.fonte, "Georgia", "+mj-lt vira a fonte do tema");
    assert_eq!(titulo.estilo.cor, "#111111", "tx1 → dk1");
    let corpo = s.elementos.iter().find(|e| e.papel == Papel::Corpo).expect("corpo");
    assert_eq!(corpo.texto, "• primeiro\n• segundo", "marcador do bodyStyle do mestre");
    let elipse = s.elementos.iter().find(|e| e.tipo == Tipo::Elipse).expect("elipse do grupo");
    assert_eq!(elipse.preenchimento, "#00ff00");
    assert!((elipse.largura - 2_000_000.0 / 12_700.0 * 4.0 / 3.0).abs() < 0.5, "grupo escala os filhos: {}", elipse.largura);
}

#[test]
fn pptx_16_10_mantem_o_tamanho_e_nao_estica() {
    // Issue #1: 16:10 (720 × 450 pt) era encaixado em 16:9 e escalado diferente em x e y.
    let bytes = pptx_minimo();
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut partes = Vec::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let mut c = String::new();
        std::io::Read::read_to_string(&mut f, &mut c).unwrap();
        partes.push((f.name().to_string(), c.replace(r#"cy="5143500"/></p:presentation>"#, r#"cy="5715000"/></p:presentation>"#)));
    }
    let partes: Vec<(&str, &str)> = partes.iter().map(|(n, c)| (n.as_str(), c.as_str())).collect();
    let ap = pptx::ler(&zip(&partes)).unwrap();
    assert_eq!(ap.tamanho(), (720.0, 450.0));
    let elipse = ap.slides[0].elementos.iter().find(|e| e.tipo == Tipo::Elipse).unwrap();
    assert!((elipse.largura / elipse.altura - 2.0).abs() < 1e-3, "proporção da forma: {} × {}", elipse.largura, elipse.altura);
    // 16:9 e 4:3 continuam no tamanho padrão.
    assert_eq!(pptx::ler(&pptx_minimo()).unwrap().dimensoes, [0.0; 2]);
}

#[test]
fn pptx_margem_interna_zero_poe_o_texto_na_borda() {
    // Issue #1: `lIns="0"` era ignorado e o texto andava 7,2 pt para dentro.
    let bytes = pptx_minimo();
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut partes = Vec::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let mut c = String::new();
        std::io::Read::read_to_string(&mut f, &mut c).unwrap();
        partes.push((f.name().to_string(), c.replace(r#"<p:txBody><a:bodyPr/><a:p><a:r><a:t>primeiro"#, r#"<p:txBody><a:bodyPr lIns="0" tIns="0"/><a:p><a:r><a:t>primeiro"#)));
    }
    let partes: Vec<(&str, &str)> = partes.iter().map(|(n, c)| (n.as_str(), c.as_str())).collect();
    let ap = pptx::ler(&zip(&partes)).unwrap();
    let corpo = ap.slides[0].elementos.iter().find(|e| e.papel == Papel::Corpo).unwrap();
    let k = 4.0 / 3.0;
    // A caixa fica onde estava; o texto começa na borda dela.
    assert!((corpo.x - 457_200.0 / 12_700.0 * k).abs() < 0.1, "x: {}", corpo.x);
    let [esq, cima, dir, _] = corpo.margens();
    assert_eq!((esq, cima), (0.0, 0.0));
    assert!((dir - 9.6).abs() < 0.01);
    let q = featherslide_core::quadro::quadro(&ap.slides[0], (960.0, 540.0), 1, &Default::default(), false);
    let linha = q.elementos.iter().find(|e| e.id == corpo.id).and_then(|e| e.linhas.first()).expect("linha do corpo");
    assert!((linha.x - corpo.x).abs() < 0.01, "o texto começa na borda: {} × {}", linha.x, corpo.x);
}

#[test]
fn pptx_grupo_girado_gira_os_filhos_em_volta_do_centro() {
    // Grupo de 2 quadrados lado a lado, girado 90°: no PowerPoint eles ficam um em cima do outro.
    let pres = format!(r#"<p:presentation {NS}><p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/></p:presentation>"#);
    let quadrado = |id: u32, x: u32| format!(r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="q"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{x}" y="0"/><a:ext cx="1270000" cy="1270000"/></a:xfrm><a:prstGeom prst="ellipse"/><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill></p:spPr></p:sp>"#);
    let slide = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:grpSp><p:grpSpPr><a:xfrm rot="5400000"><a:off x="0" y="0"/><a:ext cx="2540000" cy="1270000"/><a:chOff x="0" y="0"/><a:chExt cx="2540000" cy="1270000"/></a:xfrm></p:grpSpPr>{}{}</p:grpSp></p:spTree></p:cSld></p:sld>"#,
        quadrado(2, 0),
        quadrado(3, 1_270_000)
    );
    let bytes = zip(&[
        ("ppt/presentation.xml", &pres),
        ("ppt/_rels/presentation.xml.rels", &rels(&[("rId2", "slides/slide1.xml")])),
        ("ppt/slides/slide1.xml", &slide),
    ]);
    let ap = pptx::ler(&bytes).unwrap();
    let e: Vec<_> = ap.slides[0].elementos.iter().filter(|e| e.tipo == Tipo::Elipse).collect();
    assert_eq!(e.len(), 2);
    // 1 270 000 EMU = 100 pt. Centro do grupo (100, 50); os centros vão para (100, 0) e (100, 100).
    let centro = |i: usize| (e[i].x + e[i].largura / 2.0, e[i].y + e[i].altura / 2.0);
    let perto = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01;
    assert!(perto(centro(0), (100.0, 0.0)), "{:?}", centro(0));
    assert!(perto(centro(1), (100.0, 100.0)), "{:?}", centro(1));
    assert!((e[0].rotacao - 90.0).abs() < 0.01, "o giro do grupo passa aos filhos: {}", e[0].rotacao);
}

/// O que o Google Apresentações exporta: o desenho do tema mora no layout (setores, barras com
/// os cantos de cima redondos, branco quase transparente), a foto é recortada em elipse com
/// sombra, e a equação vem como `mc:AlternateContent` com a imagem no `mc:Fallback`.
fn pptx_com_fundo_desenhado() -> Vec<u8> {
    let pres = format!(r#"<p:presentation {NS}><p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:sldSz cx="9144000" cy="5143500"/></p:presentation>"#);
    let tema = r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:themeElements><a:clrScheme name="x"><a:dk1><a:srgbClr val="000000"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1></a:clrScheme></a:themeElements></a:theme>"#;
    let mestre = format!(r#"<p:sldMaster {NS}><p:cSld><p:spTree/></p:cSld><p:clrMap bg1="lt1" tx1="dk1"/></p:sldMaster>"#);
    let branco = r#"<a:solidFill><a:schemeClr val="lt1"><a:alpha val="9410"/></a:schemeClr></a:solidFill><a:ln><a:noFill/></a:ln>"#;
    let layout = format!(
        r#"<p:sldLayout {NS} type="title"><p:cSld><p:spTree>
<p:sp><p:nvSpPr><p:cNvPr id="2" name="t"/><p:cNvSpPr/><p:nvPr><p:ph type="ctrTitle"/></p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="100" cy="100"/></a:xfrm></p:spPr></p:sp>
<p:sp><p:nvSpPr><p:cNvPr id="3" name="pizza"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="6000000" y="500000"/><a:ext cx="1000000" cy="1000000"/></a:xfrm><a:prstGeom prst="pie"><a:avLst><a:gd fmla="val 5400000" name="adj1"/><a:gd fmla="val 16200000" name="adj2"/></a:avLst></a:prstGeom>{branco}</p:spPr></p:sp>
<p:sp><p:nvSpPr><p:cNvPr id="4" name="barra"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="7000000" y="4000000"/><a:ext cx="300000" cy="1000000"/></a:xfrm><a:prstGeom prst="round2SameRect"><a:avLst><a:gd fmla="val 50000" name="adj1"/><a:gd fmla="val 0" name="adj2"/></a:avLst></a:prstGeom>{branco}</p:spPr></p:sp>
</p:spTree></p:cSld></p:sldLayout>"#
    );
    let slide = format!(
        r#"<p:sld {NS} xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><p:cSld><p:spTree>
<p:sp><p:nvSpPr><p:cNvPr id="2" name="t"/><p:cNvSpPr/><p:nvPr><p:ph type="ctrTitle"/></p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="300000" y="2400000"/><a:ext cx="6000000" cy="1000000"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:p><a:r><a:t>Olá</a:t></a:r></a:p></p:txBody></p:sp>
<p:pic><p:nvPicPr><p:cNvPr id="5" name="foto"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="rId3"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="5900000" y="870000"/><a:ext cx="1500000" cy="1500000"/></a:xfrm><a:prstGeom prst="ellipse"><a:avLst/></a:prstGeom><a:effectLst><a:outerShdw blurRad="152400" rotWithShape="0"><a:srgbClr val="000000"><a:alpha val="80000"/></a:srgbClr></a:outerShdw></a:effectLst></p:spPr></p:pic>
<mc:AlternateContent><mc:Choice Requires="a14"><p:sp><p:nvSpPr><p:cNvPr id="6" name="eq"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="100" y="100"/><a:ext cx="100" cy="100"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:p><a:r><a:t>equação nova</a:t></a:r></a:p></p:txBody></p:sp></mc:Choice>
<mc:Fallback><p:sp><p:nvSpPr><p:cNvPr id="6" name="eq"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="100000" y="4000000"/><a:ext cx="2000000" cy="500000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:blipFill><a:blip r:embed="rId3"/><a:stretch><a:fillRect/></a:stretch></a:blipFill></p:spPr><p:txBody><a:bodyPr/><a:p><a:r><a:rPr><a:noFill/></a:rPr><a:t> </a:t></a:r></a:p></p:txBody></p:sp></mc:Fallback></mc:AlternateContent>
</p:spTree></p:cSld></p:sld>"#
    );
    zip(&[
        ("ppt/presentation.xml", &pres),
        ("ppt/_rels/presentation.xml.rels", &rels(&[("rId1", "slideMasters/slideMaster1.xml"), ("rId2", "slides/slide1.xml")])),
        ("ppt/slides/slide1.xml", &slide),
        ("ppt/slides/_rels/slide1.xml.rels", &rels(&[("rId1", "../slideLayouts/slideLayout1.xml"), ("rId3", "../media/image1.png")])),
        ("ppt/media/image1.png", "png de mentira"),
        ("ppt/slideLayouts/slideLayout1.xml", &layout),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", &rels(&[("rId1", "../slideMasters/slideMaster1.xml")])),
        ("ppt/slideMasters/slideMaster1.xml", &mestre),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels", &rels(&[("rId1", "../theme/theme1.xml")])),
        ("ppt/theme/theme1.xml", tema),
    ])
}

#[test]
fn pptx_traz_o_desenho_do_layout_a_foto_redonda_e_a_equacao() {
    let ap = pptx::ler(&pptx_com_fundo_desenhado()).unwrap();
    let s = &ap.slides[0];
    let fundo: Vec<_> = s.elementos.iter().filter(|e| e.papel == Papel::Fundo).collect();
    assert_eq!(fundo.len(), 2, "as duas formas do layout, sem a caixa de título dele");
    assert!(s.elementos.iter().take(2).all(|e| e.papel == Papel::Fundo), "o fundo fica atrás do conteúdo");
    let setor = fundo.iter().find(|e| e.tipo == Tipo::Setor).expect("pie vira setor");
    assert_eq!(setor.ajustes, vec![90.0, 270.0]);
    assert_eq!(setor.preenchimento, "#ffffff18", "branco com 9,4 % de opacidade");
    let barra = fundo.iter().find(|e| e.tipo == Tipo::RetanguloArredondado).expect("round2SameRect");
    assert_eq!(barra.ajustes, vec![0.5, 0.0], "só os cantos de cima, meia largura");
    let foto = s.elementos.iter().find(|e| e.tipo == Tipo::Imagem && e.recorte == Tipo::Elipse).expect("foto recortada em elipse");
    assert_eq!(foto.sombra.cor, "#000000cc");
    assert!((foto.sombra.desfoque - 12.0 * 4.0 / 3.0).abs() < 0.1, "152400 EMU = 12 pt: {}", foto.sombra.desfoque);
    assert_eq!(s.elementos.iter().filter(|e| e.tipo == Tipo::Imagem).count(), 2, "a equação entra pela imagem do mc:Fallback");
    assert!(!s.elementos.iter().any(|e| e.texto.contains("equação nova")));
}

#[test]
fn pptx_e_odp_ruins_dao_erro_em_portugues() {
    assert!(pptx::ler(b"nada").unwrap_err().contains(".pptx"));
    assert!(odp::ler(b"nada").unwrap_err().contains(".odp"));
    assert!(pptx::ler(&zip(&[("x.txt", "oi")])).unwrap_err().contains("PowerPoint"));
}

fn odp_minimo() -> Vec<u8> {
    let ns = r#"xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0""#;
    let estilos = format!(
        r##"<office:document-styles {ns}><office:styles><style:style style:name="Padrao-titulo" style:family="presentation"><style:text-properties fo:font-size="40pt" fo:color="#222222"/></style:style></office:styles>
<office:automatic-styles><style:page-layout style:name="PM1"><style:page-layout-properties fo:page-width="28cm" fo:page-height="15.75cm"/></style:page-layout><style:style style:name="dp1" style:family="drawing-page"><style:drawing-page-properties draw:fill="solid" draw:fill-color="#fef7e0"/></style:style></office:automatic-styles>
<office:master-styles><style:master-page style:name="Padrao" draw:style-name="dp1"/></office:master-styles></office:document-styles>"##
    );
    let conteudo = format!(
        r##"<office:document-content {ns}><office:automatic-styles>
<style:style style:name="pr1" style:family="presentation" style:parent-style-name="Padrao-titulo"><style:paragraph-properties fo:text-align="center"/></style:style>
<style:style style:name="gr1" style:family="graphic"><style:graphic-properties draw:fill="solid" draw:fill-color="#3366ff" draw:stroke="none"/></style:style>
<style:style style:name="T1" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
</office:automatic-styles><office:body><office:presentation>
<draw:page draw:name="p1" draw:master-page-name="Padrao">
<draw:frame presentation:class="title" presentation:style-name="pr1" svg:x="2cm" svg:y="1cm" svg:width="24cm" svg:height="3cm"><draw:text-box><text:p>Olá <text:span text:style-name="T1">ODP</text:span> mundo</text:p></draw:text-box></draw:frame>
<draw:frame presentation:class="outline" svg:x="2cm" svg:y="5cm" svg:width="24cm" svg:height="8cm"><draw:text-box><text:list><text:list-item><text:p>um</text:p></text:list-item><text:list-item><text:p>dois</text:p></text:list-item></text:list></draw:text-box></draw:frame>
<draw:custom-shape draw:style-name="gr1" svg:x="20cm" svg:y="10cm" svg:width="4cm" svg:height="2cm"><draw:enhanced-geometry draw:type="ellipse"/></draw:custom-shape>
<draw:line svg:x1="1cm" svg:y1="14cm" svg:x2="27cm" svg:y2="14cm"/>
<presentation:notes><draw:frame presentation:class="notes" svg:x="0cm" svg:y="0cm" svg:width="10cm" svg:height="5cm"><draw:text-box><text:p>falar devagar</text:p></draw:text-box></draw:frame></presentation:notes>
</draw:page></office:presentation></office:body></office:document-content>"##
    );
    zip(&[("mimetype", "application/vnd.oasis.opendocument.presentation"), ("content.xml", &conteudo), ("styles.xml", &estilos)])
}

#[test]
fn odp_le_caixas_listas_formas_fundo_e_notas() {
    let ap = odp::ler(&odp_minimo()).unwrap();
    assert_eq!(ap.proporcao, Proporcao::Larga);
    let s = &ap.slides[0];
    assert_eq!(s.fundo, "#fef7e0", "fundo da página mestra");
    assert_eq!(s.notas, "falar devagar");
    let t = s.elementos.iter().find(|e| e.papel == Papel::Titulo).unwrap();
    assert_eq!(t.texto, "Olá ODP mundo");
    assert_eq!(t.estilo.cor, "#222222", "cor do estilo pai");
    assert_eq!(t.estilo.alinhamento, featherslide_core::modelo::Alinhamento::Centro);
    assert!(!t.estilo.negrito, "o negrito de uma palavra não vale para a caixa");
    let corpo = s.elementos.iter().find(|e| e.papel == Papel::Corpo).unwrap();
    assert_eq!(corpo.texto, "• um\n• dois");
    let elipse = s.elementos.iter().find(|e| e.tipo == Tipo::Elipse).unwrap();
    assert_eq!(elipse.preenchimento, "#3366ff");
    assert!(elipse.contorno.is_empty());
    assert!(s.elementos.iter().any(|e| e.tipo == Tipo::Linha));
}

#[test]
fn pptx_le_fonte_embutida() {
    let pres = format!(
        r#"<p:presentation {NS}>
<p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst>
<p:sldSz cx="9144000" cy="5143500"/>
<p:embeddedFontLst>
  <p:embeddedFont>
    <p:font typeface="MinhaFonte"/>
    <p:regular r:id="rIdF1"/>
    <p:bold r:id="rIdF2"/>
  </p:embeddedFont>
</p:embeddedFontLst>
</p:presentation>"#
    );
    let rels_pres = rels(&[
        ("rId1", "slideMasters/slideMaster1.xml"),
        ("rId2", "slides/slide1.xml"),
        ("rIdF1", "fonts/fonte_reg.ttf"),
        ("rIdF2", "fonts/fonte_neg.ttf"),
    ]);
    let slide = format!(r#"<p:sld {NS}><p:cSld><p:spTree/></p:cSld></p:sld>"#);
    let mestre = format!(r#"<p:sldMaster {NS}><p:cSld><p:spTree/></p:cSld><p:clrMap bg1="lt1" tx1="dk1"/></p:sldMaster>"#);
    let layout = format!(r#"<p:sldLayout {NS} type="blank"><p:cSld><p:spTree/></p:cSld></p:sldLayout>"#);
    let tema = r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:themeElements><a:clrScheme name="x"><a:dk1><a:srgbClr val="000000"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1></a:clrScheme></a:themeElements></a:theme>"#;

    let ttf_reg = [0x00, 0x01, 0x00, 0x00, 10, 20, 30, 40];
    let ttf_neg = [0x00, 0x01, 0x00, 0x00, 50, 60, 70, 80];

    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut add = |nome: &str, bytes: &[u8]| {
        z.start_file(nome, SimpleFileOptions::default()).unwrap();
        z.write_all(bytes).unwrap();
    };
    add("ppt/presentation.xml", pres.as_bytes());
    add("ppt/_rels/presentation.xml.rels", rels_pres.as_bytes());
    add("ppt/slides/slide1.xml", slide.as_bytes());
    add("ppt/slides/_rels/slide1.xml.rels", rels(&[("rId1", "../slideLayouts/slideLayout1.xml")]).as_bytes());
    add("ppt/slideLayouts/slideLayout1.xml", layout.as_bytes());
    add("ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels(&[("rId1", "../slideMasters/slideMaster1.xml")]).as_bytes());
    add("ppt/slideMasters/slideMaster1.xml", mestre.as_bytes());
    add("ppt/slideMasters/_rels/slideMaster1.xml.rels", rels(&[("rId1", "../theme/theme1.xml")]).as_bytes());
    add("ppt/theme/theme1.xml", tema.as_bytes());
    add("ppt/fonts/fonte_reg.ttf", &ttf_reg);
    add("ppt/fonts/fonte_neg.ttf", &ttf_neg);
    let bytes_pptx = z.finish().unwrap().into_inner();

    let ap = pptx::ler(&bytes_pptx).unwrap();
    assert_eq!(ap.fontes.len(), 1);
    let f = &ap.fontes[0];
    assert_eq!(f.nome, "MinhaFonte");
    assert_eq!(f.regular.as_deref(), Some(&ttf_reg[..]));
    assert_eq!(f.negrito.as_deref(), Some(&ttf_neg[..]));
    assert!(f.italico.is_none());
}

#[test]
fn pptx_le_foto_cortada_com_src_rect() {
    let img = image::RgbaImage::new(100, 100);
    let mut png_bytes = Cursor::new(Vec::new());
    img.write_to(&mut png_bytes, image::ImageFormat::Png).unwrap();
    let png_bytes = png_bytes.into_inner();

    let pres = format!(r#"<p:presentation {NS}><p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:sldSz cx="9144000" cy="5143500"/></p:presentation>"#);
    let tema = r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:themeElements><a:clrScheme name="x"><a:dk1><a:srgbClr val="000000"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1></a:clrScheme></a:themeElements></a:theme>"#;
    let mestre = format!(r#"<p:sldMaster {NS}><p:cSld><p:spTree/></p:cSld><p:clrMap bg1="lt1" tx1="dk1"/></p:sldMaster>"#);
    let layout = format!(r#"<p:sldLayout {NS} type="blank"><p:cSld><p:spTree/></p:cSld></p:sldLayout>"#);
    let slide = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree>
<p:pic><p:nvPicPr><p:cNvPr id="5" name="foto_cortada"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>
<p:blipFill><a:blip r:embed="rId3"/><a:srcRect l="25000" t="25000" r="25000" b="25000"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>
<p:spPr><a:xfrm><a:off x="100000" y="100000"/><a:ext cx="500000" cy="500000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
</p:pic>
</p:spTree></p:cSld></p:sld>"#
    );

    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut add = |nome: &str, bytes: &[u8]| {
        z.start_file(nome, SimpleFileOptions::default()).unwrap();
        z.write_all(bytes).unwrap();
    };
    add("ppt/presentation.xml", pres.as_bytes());
    add("ppt/_rels/presentation.xml.rels", rels(&[("rId1", "slideMasters/slideMaster1.xml"), ("rId2", "slides/slide1.xml")]).as_bytes());
    add("ppt/slides/slide1.xml", slide.as_bytes());
    add("ppt/slides/_rels/slide1.xml.rels", rels(&[("rId1", "../slideLayouts/slideLayout1.xml"), ("rId3", "../media/foto.png")]).as_bytes());
    add("ppt/slideLayouts/slideLayout1.xml", layout.as_bytes());
    add("ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels(&[("rId1", "../slideMasters/slideMaster1.xml")]).as_bytes());
    add("ppt/slideMasters/slideMaster1.xml", mestre.as_bytes());
    add("ppt/slideMasters/_rels/slideMaster1.xml.rels", rels(&[("rId1", "../theme/theme1.xml")]).as_bytes());
    add("ppt/theme/theme1.xml", tema.as_bytes());
    add("ppt/media/foto.png", &png_bytes);
    let pptx_bytes = z.finish().unwrap().into_inner();

    let ap = pptx::ler(&pptx_bytes).unwrap();
    assert_eq!(ap.slides.len(), 1);
    let foto = ap.slides[0].elementos.iter().find(|e| e.tipo == Tipo::Imagem).expect("foto presente");
    let (_, decodificado) = featherslide_io::base64::de_data_url(&foto.imagem).unwrap();
    let img_cortada = image::load_from_memory(&decodificado).unwrap();
    assert_eq!(img_cortada.width(), 50);
    assert_eq!(img_cortada.height(), 50);
}


#[test]
fn pptx_gravado_le_de_volta_igual() {
    use featherslide_core::sessao::Sessao;
    use featherslide_core::Formatacao;
    let mut s = Sessao::default();
    let id = s.apresentacao().slides[0].elementos[0].id;
    s.texto_definir(id, "Título & <acentuação>").unwrap();
    s.notas_definir("falar do orçamento\nsegunda linha").unwrap();
    s.slide_novo("em_branco").unwrap();
    s.elemento_inserir(Tipo::Elipse, Some([100.0, 50.0, 200.0, 100.0]), None).unwrap();
    s.formatar(Formatacao::Preenchimento("#34a853".into())).unwrap();
    const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    s.elemento_inserir(Tipo::Imagem, Some([400.0, 300.0, 120.0, 90.0]), Some(PNG.into())).unwrap();
    let g = pptx::gravar::gravar(s.apresentacao()).unwrap();
    assert_eq!(g.de_fora, 0);
    let lida = pptx::ler(&g.bytes).unwrap();
    assert_eq!(lida.slides.len(), 2);
    assert_eq!(lida.tamanho(), s.apresentacao().tamanho());
    assert!(lida.slides[0].elementos.iter().any(|e| e.texto == "Título & <acentuação>" && e.papel == Papel::Titulo), "{:?}", lida.slides[0].elementos.iter().map(|e| &e.texto).collect::<Vec<_>>());
    assert_eq!(lida.slides[0].notas, "falar do orçamento\nsegunda linha");
    let elipse = lida.slides[1].elementos.iter().find(|e| e.tipo == Tipo::Elipse).expect("elipse");
    assert_eq!((elipse.x, elipse.y, elipse.largura, elipse.altura), (100.0, 50.0, 200.0, 100.0));
    assert_eq!(elipse.preenchimento, "#34a853");
    assert!(lida.slides[1].elementos.iter().any(|e| e.tipo == Tipo::Imagem && (e.x - 400.0).abs() < 0.01));
}

#[test]
fn pptx_gravado_leva_tabela_e_grafico_nativos() {
    use featherslide_core::sessao::Sessao;
    let mut s = Sessao::default();
    s.tabela_inserir(3, 2).unwrap();
    s.grafico_inserir("colunas").unwrap();
    let g = pptx::gravar::gravar(s.apresentacao()).unwrap();
    assert_eq!(g.de_fora, 0);
    let mut z = zip::ZipArchive::new(Cursor::new(g.bytes)).unwrap();
    let mut ler = |nome: &str| { let mut c = String::new(); std::io::Read::read_to_string(&mut z.by_name(nome).unwrap(), &mut c).unwrap(); c };
    let slide = ler("ppt/slides/slide1.xml");
    assert!(slide.contains("<a:tbl>") && slide.contains("<c:chart "), "{slide}");
    let grafico = ler("ppt/charts/chart1.xml");
    assert!(grafico.contains("<c:barChart>") && grafico.contains("<c:numLit>") && grafico.contains("<c:v>Série 1</c:v>"), "{grafico}");
    assert!(ler("[Content_Types].xml").contains("/ppt/charts/chart1.xml"));
    assert!(ler("ppt/slides/_rels/slide1.xml.rels").contains("../charts/chart1.xml"));
}
