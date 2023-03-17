LibreOffice plugins
==============================================

Here I write down what I was able to piece together about LO plugins while writing this one, since knowledge of this stuff appears to die out.

## Archeaological methods

 - Use [API doc](https://api.libreoffice.org/docs/idl/ref/index.html)
 - Unzip .oxt files downloaded from [LibreOffice Extensions](https://extensions.libreoffice.org/) and look at code, copy/paste and adapt where necessary.
 - Inspect certain objects directly in LO using `Tools > Development Tools`.

## Elements of understanding

  - All LO stuff is a UNO object.
  - Organized in hierarchies, which the doc allows one to browse.
  - In Python, one can (i) sub-class some of these objects, (ii) call functions which interact with such objects.
  - Package is a zipped file renamed to .oxt containing among other stuff:
     * `description.xml` : some info about the package itself (name, author, license, etc.)
     * `META-INF/manifest.xml` : an XML file describing each file in the oxt folder and their function
