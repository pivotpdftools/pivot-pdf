# Issue 1: Basic PDF Creation
## Description
Create enough code in the pdf-core to produce a minimal pdf that can be opened by any pdf viewer and have no errors. A generated pdf will use a default version 1.7.

IMPORTANT: The design must support the ability for a consumer to flush to the disk after x number of pages written. This is to allow low memory usage by allowing page data to be freed. However, for small files with the need to stream the result, file name can be omitted and disk flushing not an option in this case.

The basic design sketch in psuedo code:
```ts
let p = new PdfDocument()
p.beginDocument(file: "example.pdf")
p.setInfo("Creator", "rust-pdf")
p.setInfo("Title", "A Test Document")

//width/height as pt units.
p.beginPage(width: 612, height: 792, {
    origin: Origin.BottomLeft //this will be default
})

p.place_text("Hello", x: 20, y: 20) //will use default helvetica as other fonts are not yet supported

p.engPage() //not sure if this is needed

p.endDocument() //not sure if this is needed but we need some way to flush the content to the file specificed in beginDocument
```
There should be a way to generate a document like sketched above and output it somewhere so the user can view it.

## Tasks
- [x] Fix Cargo.toml and create project skeleton
- [x] Implement PDF object types (objects.rs)
- [x] Implement PDF binary writer (writer.rs)
- [x] Implement high-level API (document.rs)
- [x] Integration test — produce and validate a real PDF

## Status
complete

---

# Issue 2: Improve test organization
## Description
The previous Issue ogranized tests with some in the tests directory and others in the same file as the code. All tests should be organized in the tests directory.

## Tasks
- [x] Identify the tests to move
- [x] Move the tests

## Status
complete

---

# Issue 3: TextFlows
## Description
A critical feature of this library is TextFlows. TextFlows:
- Allow text to be placed within the specified boundries using different methods:
  - None (Default) - will place the text until the bounding box is full
  - Shrink - scale the font down until it fits within the bounding box or reaches a minimum value. Note, we are currently working only with Helvetica font but any font should be considered
  - Clip - Clip the
- Allow text styling such as "The __bold__ brown fox"

The design will expand over time but this is intended to be the minimum implementation to demonstrate whether it works.

Psuedo Code Usage Sketch
```rs
let path = "sample_output.pdf";
let mut doc = PdfDocument::create(path).unwrap();
doc.set_info("Creator", "rust-pdf");
doc.set_info("Title", "A Test Document");

let mut text_flow = doc.create_textflow("Lorum Ipsum ...Long text to flow 2 pages")
text_flow.add_textflow("bold text", {font_weigth: 900})
text_flow.add_textflow("a little more normal text. The end.")

loop {
    //the start of each loop creates a new page
    doc.begin_page(612.0, 792.0);

    //The fit_textflow result will help inform the loop what to do
    let result = doc.fit_textflow(text_flow, x: 72.0, y: 720.0, width: 100, height: 100);

    //Stop: All the text has been fit and there is no more
    //BoxFull: The box is full and not all the text has been placed
    //BoxEmpty: 
    match result {
        FitResult::Stop => break,
        FitResult::BoxFull => continue,
        FitResult::BoxEmpty => {
            eprintln!("Warning: bounding box too small");
            break;
        }
    }
}
doc.end_page().unwrap();
doc.end_document().unwrap();
```

## Tasks
- [x] Evaluate the psuedo code above and offer design suggestions.
- [x] Plan the remaining steps
- [x] Create fonts.rs with Helvetica metrics
- [x] Add Helvetica-Bold font to document
- [x] Create textflow.rs with data structures
- [x] Implement word wrapping and content stream generation
- [x] Integrate with PdfDocument via fit_textflow
- [x] End-to-end tests

## Status
complete
