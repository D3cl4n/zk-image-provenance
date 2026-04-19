import struct
import zlib
import io
from PIL import Image

'''
- This script aims to demonstrate editor functionality
- The editor recomputes the hash Poseidon(r||g||b||exif) to send to verifier as separate input
- TODO: rewrite this to read image bytes directly and only greyscale pixels from IDATA chunk
    - need to save all bytes 
'''
    

# class for all functionality related to the editor
class Editor:
    def __init__(self, image_path):
        self.image_path = image_path
        self.chunks = {} # keys is chunk types, value is data
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.r = []
        self.g = []
        self.b = []
        self.greyscale_pixels = []
        self.target = "greyscale.png"


    # helper function to read a chunk
    def read_chunk(self, f):
        chunk_length, chunk_type = struct.unpack(">I4s", f.read(8))
        print(f"[*] Chunk type: {chunk_type}, chunk length {chunk_length}")
        chunk_data = f.read(chunk_length)
        expected_crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff
        actual_crc, = struct.unpack(">I", f.read(4))

        # check the crc for data corruption
        assert expected_crc == actual_crc

        return chunk_type, chunk_data


    # parse the png and save chunks in separate buffers, consolidate pixels to one buffer
    # follows this guide https://pyokagan.name/blog/2019-10-14-png/ 
    # TODO: change this to be able to concatenate multiple IDAT blocks
    def parse_png(self):
        print("[*] Parsing png chunks")

        png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        with open(self.image_path, "rb") as f:
            actual_signature = f.read(8)
            assert actual_signature == png_signature

            # parse all chunks out of the file
            while True:
                chunk_type, chunk_data = self.read_chunk(f)
                self.chunks[chunk_type] = chunk_data
                if chunk_type == b"IEND":
                    break

    
    # greyscale the pixels in the IDAT chunk
    def greyscale(self):
        print("[*] Greyscaling pixels in png")
        image = Image.open(self.image_path).convert("L")
        temp = io.BytesIO()
        image.save(temp, format="PNG")
        temp.seek(0)

        new_chunks = {}
        temp.read(8) # skip the png signature


    # reassamble and save the edited png based on dictionary of chunks
    def reassemble_png(self):
        pass

            
# main function
def main():
    # editor functionality
    editor = Editor("original.png")
    editor.parse_png()
    editor.flip_color_byte()
    editor.greyscale()


if __name__ == '__main__':
    main()