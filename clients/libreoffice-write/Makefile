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
		images/*.png \
		src/* \
		config/* \
		dialogs/* \
		package/license.txt 

install:
	unopkg validate $(OUTPUT)
	unopkg add $(OUTPUT)

validate:
	unopkg validata $(OUTPUT)
