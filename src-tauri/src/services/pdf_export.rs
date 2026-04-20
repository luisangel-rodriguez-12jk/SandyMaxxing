use printpdf::{BuiltinFont, IndirectFontRef, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference};

use crate::ai::plan_generator::PlanResult;
use crate::db::models::{Measurement, ShoppingItem};
use crate::error::{AppError, AppResult};

fn pdf_err<E: std::fmt::Debug>(e: E) -> AppError {
    AppError::Other(format!("pdf: {e:?}"))
}

fn finish(doc: PdfDocumentReference) -> AppResult<Vec<u8>> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = std::io::BufWriter::new(cursor);
    doc.save(&mut writer).map_err(pdf_err)?;
    let cursor = writer.into_inner().map_err(pdf_err)?;
    Ok(cursor.into_inner())
}

const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 18.0;

struct Cursor {
    y: f32,
}

impl Cursor {
    fn new() -> Self {
        Self { y: PAGE_H - MARGIN }
    }
    fn line(&mut self, layer: &PdfLayerReference, font: &IndirectFontRef, size: f32, text: &str) {
        layer.use_text(text, size, Mm(MARGIN), Mm(self.y), font);
        self.y -= size * 0.55 + 1.5;
    }
    fn gap(&mut self, h: f32) {
        self.y -= h;
    }
}

fn new_page(doc: &PdfDocumentReference) -> (printpdf::PdfPageIndex, printpdf::PdfLayerIndex) {
    doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "capa")
}

pub fn plan_to_pdf(plan: &PlanResult, title: &str) -> AppResult<Vec<u8>> {
    let (doc, page, layer) = PdfDocument::new(title, Mm(PAGE_W), Mm(PAGE_H), "capa");
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(pdf_err)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(pdf_err)?;
    let mut current_layer = doc.get_page(page).get_layer(layer);
    let mut cur = Cursor::new();
    cur.line(&current_layer, &font, 18.0, title);
    cur.gap(2.0);
    for day in &plan.days {
        if cur.y < 40.0 {
            let (p, l) = new_page(&doc);
            current_layer = doc.get_page(p).get_layer(l);
            cur = Cursor::new();
        }
        cur.line(&current_layer, &font, 14.0, &day.day);
        for meal in &day.meals {
            cur.line(
                &current_layer,
                &font,
                11.0,
                &format!("  {} — {}", meal.meal_type, meal.name),
            );
            cur.line(&current_layer, &font_regular, 9.0, &format!("    {}", meal.instructions));
            for ing in &meal.ingredients {
                cur.line(
                    &current_layer,
                    &font_regular,
                    9.0,
                    &format!("    • {} {} {}", ing.quantity, ing.unit, ing.name),
                );
            }
            for up in &meal.per_user_portions {
                cur.line(
                    &current_layer,
                    &font_regular,
                    9.0,
                    &format!("    ({}) {}", up.user, up.notes),
                );
            }
            if cur.y < 40.0 {
                let (p, l) = new_page(&doc);
                current_layer = doc.get_page(p).get_layer(l);
                cur = Cursor::new();
            }
        }
        cur.gap(2.0);
    }
    finish(doc)
}

pub fn shopping_to_pdf(items: &[ShoppingItem], title: &str) -> AppResult<Vec<u8>> {
    let (doc, page, layer) = PdfDocument::new(title, Mm(PAGE_W), Mm(PAGE_H), "capa");
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(pdf_err)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(pdf_err)?;
    let mut current_layer = doc.get_page(page).get_layer(layer);
    let mut cur = Cursor::new();
    cur.line(&current_layer, &font, 18.0, title);
    cur.gap(2.0);
    let mut current_group = "".to_string();
    for item in items {
        if item.group_name != current_group {
            current_group = item.group_name.clone();
            cur.gap(1.0);
            cur.line(&current_layer, &font, 12.0, &current_group);
        }
        cur.line(
            &current_layer,
            &font_regular,
            10.0,
            &format!("  • {} — {:.2} {}", item.name, item.quantity, item.unit),
        );
        if cur.y < 20.0 {
            let (p, l) = new_page(&doc);
            current_layer = doc.get_page(p).get_layer(l);
            cur = Cursor::new();
        }
    }
    finish(doc)
}

pub fn measurements_to_pdf(
    user_name: &str,
    measurements: &[Measurement],
) -> AppResult<Vec<u8>> {
    let title = format!("Mediciones — {user_name}");
    let (doc, page, layer) = PdfDocument::new(&title, Mm(PAGE_W), Mm(PAGE_H), "capa");
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(pdf_err)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(pdf_err)?;
    let mut current_layer = doc.get_page(page).get_layer(layer);
    let mut cur = Cursor::new();
    cur.line(&current_layer, &font, 16.0, &title);
    cur.gap(2.0);
    cur.line(
        &current_layer,
        &font,
        10.0,
        "Fecha   Peso   Espalda   Cintura   Abdomen   Cadera",
    );
    for m in measurements {
        let row = format!(
            "{}   {:>5}   {:>7}   {:>7}   {:>7}   {:>6}",
            m.date,
            m.weight.map(|v| format!("{v:.1}")).unwrap_or_else(|| "-".into()),
            m.back_cm.map(|v| format!("{v:.1}")).unwrap_or_else(|| "-".into()),
            m.waist_cm.map(|v| format!("{v:.1}")).unwrap_or_else(|| "-".into()),
            m.abdomen_cm.map(|v| format!("{v:.1}")).unwrap_or_else(|| "-".into()),
            m.hip_cm.map(|v| format!("{v:.1}")).unwrap_or_else(|| "-".into()),
        );
        cur.line(&current_layer, &font_regular, 9.0, &row);
        if cur.y < 20.0 {
            let (p, l) = new_page(&doc);
            current_layer = doc.get_page(p).get_layer(l);
            cur = Cursor::new();
        }
    }
    finish(doc)
}

