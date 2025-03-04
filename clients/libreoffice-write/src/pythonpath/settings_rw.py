import logging
from utils import getConfigurationAccess

default_settings = {
	"MathsPreviewPath"  : "~/bin/maths_preview",
	"MathsFont"         : None,
	"CustomCommandFile" : None,
	"TextAsText"        : True,
}



def validate_string(v):
	return isinstance(v, str) and v.strip() != ""

def validate_checkox(cb):
	return isinstance(cb, bool)


settings_validator = {
	"MathsPreviewPath"  : validate_string,
	"MathsFont"         : validate_string,
	"CustomCommandFile" : validate_string,
	"TextAsText"        : validate_checkox,
}

def save_settings(key, new_settings):
	validate_settings(new_settings)

	alpha_ordered_setting_names  = tuple(sorted(list(new_settings.keys())))
	corresponding_setting_values = tuple(new_settings[name] for name in alpha_ordered_setting_names)

	try:
		writer = getConfigurationAccess(key, True)
		writer.setPropertyValues(
			alpha_ordered_setting_names, 
			corresponding_setting_values
		)
		writer.commitChanges()
	except Exception as e:
		raise e

def read_settings(key):
	reader = getConfigurationAccess(key)
	if reader is None:
		return None
	names = reader.ElementNames
	values = reader.getPropertyValues(names)
	settings = {k: v for k, v in zip(names, values)}

	validate_settings(settings)
	logging.debug("read settings: {}".format(settings))
	return settings

def validate_settings(settings):
	# All invalid values are set to default values
	extra_keys = [key for key in settings.keys() if key not in default_settings]
	for extra_key in extra_keys:
		del settings[extra_key]

	for key, default_value in default_settings.items():
		if key not in settings or not settings_validator[key](settings[key]):
			settings[key] = default_value			

