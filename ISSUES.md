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

