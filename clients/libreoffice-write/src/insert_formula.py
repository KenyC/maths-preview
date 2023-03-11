# coding: utf-8
from __future__ import unicode_literals
import uno
import unohelper
from com.sun.star.beans import PropertyValue
from com.sun.star.awt import Size
from com.sun.star.text.ControlCharacter import PARAGRAPH_BREAK, LINE_BREAK
from com.sun.star.awt.FontWeight import BOLD as FW_BOLD
import enum
import logging
import time
import subprocess
import tempfile
import os
import uuid

FORMATS = [
	"svg",
	"png"
]
PATH_EXE = "/home/keny/bin/maths_preview"

def main(*args):
	# create temporary file path
	path = os.path.join(tempfile.gettempdir(), str(uuid.uuid4()))

	# Recover document
	desktop = XSCRIPTCONTEXT.getDesktop()
	doc = desktop.getCurrentComponent()

	# Get graphics provider (loads image files and stuff)
	service_manager  = uno.getComponentContext().getServiceManager() 
	graphic_provider = service_manager.createInstance('com.sun.star.graphic.GraphicProvider')

	# Get cursor
	# cursor = doc.Text.createTextCursor()
	cursor = doc.CurrentController.ViewCursor	
	char_height = cursor.CharHeight

	# Start program
	result = subprocess.run([
		PATH_EXE, 
		"-s", str(char_height),
		"-f", "svg", 
		"-o", path
	], 
		stdout = subprocess.PIPE, 
		stderr = subprocess.PIPE,
	)
	print(result.stdout.decode("utf-8"))
	print(result.stderr.decode("utf-8"))
	if result.returncode != 0:
		print("ERROR: maths_preview didn't return positive")
		return

	# Paste image
	add_embedded_image(
		cursor,
		graphic_provider, 
		doc,
		path,
	)


def add_embedded_image(cursor, graphic_provider, doc, path, dpi = 1.0, width=None, height=None, paraadjust=None):
	# TODO : dpi should be guessed from buffer
	# scale = 1000 * 2.54 / float(dpi)
	scale = 10 * 2.54 / float(dpi) # this seems like the good ratio, I have no idea why!


	try:
		file_url = unohelper.systemPathToFileUrl(path)
		graphic = graphic_provider.queryGraphic((PropertyValue('URL', 0, file_url, 0), ))
		if graphic is None:
			print("No file!")
			return
			
		if graphic.SizePixel is None:
			# Then we're likely dealing with vector graphics. Then we try to
			# get the "real" size, which is enough information to
			# determine the aspect ratio
			original_size = graphic.Size100thMM
		else:
			original_size = graphic.SizePixel
		print("graphic.Size100thMM", graphic.Size100thMM.Width, graphic.Size100thMM.Height)
		print("graphic.SizePixel",   graphic.SizePixel.Width,   graphic.SizePixel.Height)
		graphic_object_shape = doc.createInstance('com.sun.star.drawing.GraphicObjectShape')
		graphic_object_shape.Graphic = graphic
		if width and height:
			size = Size(int(width * scale), int(height * scale))
		elif width:
			size = Size(int(width * scale), int((float(width)/original_size.Width) * original_size.Height * scale))
		elif height:
			size = Size(int((float(height)/original_size.Height) * original_size.Width * scale), int(height * scale))
		else:
			size = Size(int(original_size.Width * scale), original_size.Height * scale)
		graphic_object_shape.setSize(size)
		# doc.Text.insertTextContent(cursor, graphic_object_shape, False)
		text_graphic_object = doc.createInstance("com.sun.star.text.TextGraphicObject")
		text_graphic_object.Graphic = graphic_object_shape.Graphic
		text_graphic_object.setSize(size)

		if paraadjust:
			oldparaadjust = cursor.ParaAdjust
			cursor.ParaAdjust = paraadjust
		doc.Text.insertTextContent(cursor, text_graphic_object, False)
		# os.unlink(url)
		if paraadjust:
			cursor.ParaAdjust = oldparaadjust
	except Exception as e:
		print(e)



