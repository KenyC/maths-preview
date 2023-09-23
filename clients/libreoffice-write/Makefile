NAME=InsertFromMathsPreview
OUTPUT=$(NAME).oxt

all:	clean zip install

clean:
	-unopkg remove $(OUTPUT)
	rm -f $(OUTPUT)
	
zip:
	zip -MM -r $(OUTPUT) \
		description.xml \
		META-INF/manifest.xml \
		Addons.xcu \
		ToolbarState.xcu \
		images/*.png \
		src/* \
		config/* \
		dialogs/* \
		package/LICENSE

install:
	unopkg validate $(OUTPUT)
	unopkg add $(OUTPUT)

validate:
	unopkg validata $(OUTPUT)
