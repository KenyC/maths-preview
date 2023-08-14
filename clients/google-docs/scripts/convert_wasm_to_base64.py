# ChatGPT-generated script
import argparse
import base64

def generate_js_file(input_filename, output_filename):
    with open(input_filename, 'rb') as input_file:
        binary_data = input_file.read()
        base64_data = base64.b64encode(binary_data).decode('utf-8')

    js_content = f'const wasm_bytecode = "{base64_data}";'

    with open(output_filename, 'w') as output_file:
        output_file.write(js_content)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Generate a JavaScript file with base64-encoded binary data')
    parser.add_argument('input_file', help='Path to the input binary file')
    parser.add_argument('output_file', help='Path to the output JavaScript file')
    args = parser.parse_args()

    generate_js_file(args.input_file, args.output_file)
