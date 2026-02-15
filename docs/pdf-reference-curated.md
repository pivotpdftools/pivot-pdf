# Cureated PDF reference PDF 32000-1:2008
Clipped from the spec to provide context for AI.
© Adobe Systems Incorporated 2008 – All rights reserved

### 7.1 General
This clause covers everything about the syntax of PDF at the object, file, and document level. It sets the stage
for subsequent clauses, which describe how the contents of a PDF file are interpreted as page descriptions,
interactive navigational aids, and application-level logical structure.

PDF syntax is best understood by considering it as four parts, as shown in Figure 1:

- Objects. A PDF document is a data structure composed from a small set of basic types of data objects. Sub-clause 7.2, "Lexical Conventions," describes the character set used to write objects and other syntactic elements. Sub-clause 7.3, "Objects," describes the syntax and essential properties of the objects. Sub-clause 7.3.8, "Stream Objects," provides complete details of the most complex data type, the stream object.
- File structure. The PDF file structure determines how objects are stored in a PDF file, how they are accessed, and how they are updated. This structure is independent of the semantics of the objects. Sub-clause 7.5, "File Structure," describes the file structure. Sub-clause 7.6, "Encryption," describes a file-level mechanism for protecting a document’s contents from unauthorized access.
- Document structure. The PDF document structure specifies how the basic object types are used to
represent components of a PDF document: pages, fonts, annotations, and so forth. Sub-clause 7.7,
"Document Structure," describes the overall document structure; later clauses address the detailed
semantics of the components.
- Content streams. A PDF content stream contains a sequence of instructions describing the appearance of a page or other graphical entity. These instructions, while also represented as objects, are conceptually distinct from the objects that represent the document structure and are described separately. Sub-clause 7.8, "Content Streams and Resources," discusses PDF content streams and their associated resources.

### 7.4 Filters
#### 7.4.1 General
Stream filters are introduced in 7.3.8, "Stream Objects." An option when reading stream data is to decode it using a filter to produce the original non-encoded data. Whether to do so and which decoding filter or filters to use may be specified in the stream dictionary.

PDF supports a standard set of filters that fall into two main categories:
- ASCII filters enable decoding of arbitrary binary data that has been encoded as ASCII text (see 7.2, "Lexical Conventions," for an explanation of why this type of encoding might be useful).
- Decompression filters enable decoding of data that has been compressed. The compressed data shall be in binary format, even if the original data is ASCII text.

| FILTER name  | Parameters | Description                                  |
|--------------|------------|----------------------------------------------|
| FlateDecode  |    yes     | (PDF 1.2) Decompresses data encoded using    |
|              |            | the zlib/deflate compression method,         |
|              |            | reproducing the original text or binary data |


## 9 Text
### 9.1 General
This clause describes the special facilities in PDF for dealing with text—specifically, for representing characters with glyphs from fonts. A glyph is a graphical shape and is subject to all graphical manipulations, such as coordinate transformation. Because of the importance of text in most page descriptions, PDF provides higher-level facilities to describe, select, and render glyphs conveniently and efficiently.

The first sub-clause is a general description of how glyphs from fonts are painted on the page. Subsequent sub-clauses cover these topics in detail:
- Text state. A subset of the graphics state parameters pertain to text, including parameters that select the font, scale the glyphs to an appropriate size, and accomplish other graphical effects.
- Text objects and operators. The text operators specify the glyphs to be painted, represented by string objects whose values shall be interpreted as sequences of character codes. A text object encloses a sequence of text operators and associated parameters.
- Font data structures. Font dictionaries and associated data structures provide information that a
conforming reader needs to interpret the text and position the glyphs properly. The definitions of the glyphs themselves shall be contained in font programs, which may be embedded in the PDF file, built into a conforming reader, or obtained from an external font file.

### 9.2 Organization and Use of Fonts
#### 9.2.1 General
A character is an abstract symbol, whereas a glyph is a specific graphical rendering of a character.

EXAMPLE 1 The glyphs A, A, and A are renderings of the abstract “A” character.

NOTE 1 Historically these two terms have often been used interchangeably in computer typography (as evidenced by the names chosen for some PDF dictionary keys and PostScript operators), but advances in this area have made the distinction more meaningful. Consequently, this standard distinguishes between characters and glyphs, though with some residual names that are inconsistent.
Glyphs are organized into fonts. A font defines glyphs for a particular character set.

EXAMPLE 2 The Helvetica and Times fonts define glyphs for a set of standard Latin characters.
A font for use with a conforming reader is prepared in the form of a program. Such a font program shall be written in a special-purpose language, such as the Type 1, TrueType, or OpenType font format, that is understood by a specialized font interpreter.
In PDF, the term font refers to a font dictionary, a PDF object that identifies the font program and contains additional information about it. There are several different font types, identified by the Subtype entry of the font dictionary.
For most font types, the font program shall be defined in a separate font file, which may be either embedded in a PDF stream object or obtained from an external source. The font program contains glyph descriptions that generate glyphs.
A content stream paints glyphs on the page by specifying a font dictionary and a string object that shall be interpreted as a sequence of one or more character codes identifying glyphs in the font. This operation is called showing the text string; the text strings drawn in this way are called show strings. The glyph description consists of a sequence of graphics operators that produce the specific shape for that character in this font. To render a glyph, the conforming reader executes the glyph description.


NOTE 2 Programmers who have experience with scan conversion of general shapes may be concerned about the amount of computation that this description seems to imply. However, this is only the abstract behaviour of glyph descriptions and font programs, not how they are implemented. In fact, an efficient implementation can be achieved through careful caching and reuse of previously rendered glyphs.

#### 9.2.2 Basics of Showing Text
EXAMPLE 1 This example illustrates the most straightforward use of a font. The text ABC is placed 10 inches from the bottom of the page and 4 inches from the left edge, using 12-point Helvetica.

```
BT
  /F13 12 Tf
  288 720 Td
  ( ABC ) Tj
ET
```

The five lines of this example perform these steps:
a) Begin a text object.
b) Set the font and font size to use, installing them as parameters in the text state. In this case, the font
resource identified by the name F13 specifies the font externally known as Helvetica.
c) Specify a starting position on the page, setting parameters in the text object.
d) Paint the glyphs for a string of characters at that position.
e) End the text object.

These paragraphs explain these operations in more detail.
To paint glyphs, a content stream shall first identify the font to be used. The Tf operator shall specify the name of a font resource—that is, an entry in the Font subdictionary of the current resource dictionary. The value of that entry shall be a font dictionary. The font dictionary shall identify the font’s externally known name, such as Helvetica, and shall supply some additional information that the conforming reader needs to paint glyphs from that font. The font dictionary may provide the definition of the font program itself.

NOTE 1 The font resource name presented to the Tf operator is arbitrary, as are the names for all kinds of resources. It bears no relationship to an actual font name, such as Helvetica.

EXAMPLE 2 This Example illustrates an excerpt from the current page’s resource dictionary, which defines the font dictionary that is referenced as F13 (see EXAMPLE 1 in this sub-clause).

```
/Resources
  << /Font << /F13 23 0 R >>
  >>
23 0 obj
  << /Type /Font
     /Subtype /Type1
     /BaseFont /Helvetica
  >>
endobj
```

A font defines the glyphs at one standard size. This standard is arranged so that the nominal height of tightly spaced lines of text is 1 unit. In the default user coordinate system, this means the standard glyph size is 1 unit in user space, or 1 ⁄ 72 inch. Starting with PDF 1.6, the size of this unit may be specified as greater than 1 ⁄ 72 inch by means of the UserUnit entry of the page dictionary; see Table 30. The standard-size font shall then be scaled to be usable. The scale factor is specified as the second operand of the Tf operator, thereby setting the text font size parameter in the graphics state. EXAMPLE 1 in this sub-clause establishes the Helvetica font with a 12-unit size in the graphics state.

Once the font has been selected and scaled, it may be used to paint glyphs. The Td operator shall adjust the translation components of the text matrix, as described in 9.4.2, "Text-Positioning Operators". When executed for the first time after BT, Td shall establish the text position in the current user coordinate system. This determines the position on the page at which to begin painting glyphs.

The Tj operator shall take a string operand and shall paint the corresponding glyphs, using the current font and other text-related parameters in the graphics state.

NOTE 2 The Tj operator treats each element of the string (an integer in the range 0 to 255) as a character code (see EXAMPLE 1 in this sub-clause).

Each byte shall select a glyph description in the font, and the glyph description shall be executed to paint that glyph on the page. This is the behaviour of Tj for simple fonts, such as ordinary Latin text fonts. Interpretation of the string as a sequence of character codes is more complex for composite fonts, described in 9.7, "Composite Fonts".

What these steps produce on the page is not a 12-point glyph, but rather a 12-unit glyph, where the unit size shall be that of the text space at the time the glyphs are rendered on the page. The actual size of the glyph shall be determined by the text matrix (Tm ) in the text object, several text state parameters, and the current transformation matrix (CTM) in the graphics state; see 9.4.4, "Text Space Details".

EXAMPLE 3 If the text space is later scaled to make the unit size 1 centimeter, painting glyphs from the same 12-unit font generates results that are 12 centimeters high.