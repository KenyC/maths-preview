NAME=insert_from_maths_preview
OUTPUT=$(NAME).oxt

all:	clean zip install

clean:
	-unopkg remove $(OUTPUT)
	rm -f $(OUTPUT)
	
zip:
	zip -r $(OUTPUT) \
		description.xml \
		META-INF/manifest.xml \
		Addons.xcu \
		src/insert_formula.py \
		package/license.txt 

install:
	unopkg add $(OUTPUT)

validate:
	unopkg validata $(OUTPUT)