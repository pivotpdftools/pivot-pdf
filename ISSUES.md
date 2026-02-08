# Issue 1: Basic PDF Creation
## Description
Create enough code in the pdf-core to produce a minimal pdf that can be opened by any pdf viewer and have no errors. A generated pdf will use a default version 1.7.
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

## Tasks
TODO, create the tasks

## Status
ready

---
