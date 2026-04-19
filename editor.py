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
        self.greyscale_chunks = {} # for edited chunks only
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.r = []
        self.g = []
        self.b = []
        self.png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        self.target = "greyscale.png"


    # helper function to read a chunk
    def read_chunk(self, f):
        chunk_length, chunk_type = struct.unpack(">I4s", f.read(8))
        chunk_data = f.read(chunk_length)
        expected_crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff
        actual_crc, = struct.unpack(">I", f.read(4))

        # check the crc for data corruption
        assert expected_crc == actual_crc

        return chunk_type, chunk_data


    # parse the png and save chunks in separate buffers, consolidate pixels to one buffer
    # follows this guide https://pyokagan.name/blog/2019-10-14-png/ 
    # TODO: change this to be able to concatenate multiple IDAT blocks
    # TODO: make this abstract to use with the grey image as well
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

        print(f"[*] Read chunks: {list(self.chunks.keys())}")

    
    # greyscale the pixels in the IDAT chunk
    def greyscale(self):
        print("[*] Greyscaling pixels in png")
        image = Image.open(self.image_path).convert("RGB")
        width, height = image.size
        pixels = image.load()
        grey_image = Image.new("L", (width, height))
        grey_pixels = grey_image.load()

        for y in range(height):
            for x in range(width):
                r, g, b = pixels[x, y]
                grey_val = (self.r_coeff * r + self.g_coeff * g + self.b_coeff * b) // 100 # using integer coefficients not floats
                grey_pixels[x, y] = grey_val

        # save the greyscaled image to a temporary buffer to parse out IDAT chunk
        temp_buffer = io.BytesIO()
        grey_image.save(temp_buffer, format="PNG")
        temp_buffer.seek(0)

        # read out the new IHDR and IDAT chunks for reassembly
        temp_buffer.read(8) # skip the png signature
        while True:
            chunk_length_bytes = temp_buffer.read(4)
            chunk_length = struct.unpack(">I", chunk_length_bytes)[0]
            chunk_type = temp_buffer.read(4)
            chunk_data = temp_buffer.read(chunk_length)
            chunk_crc = temp_buffer.read(4)

            self.greyscale_chunks[chunk_type] = chunk_length_bytes + chunk_type + chunk_data + chunk_crc

            if chunk_type == b"IEND":
                break

        print(f"[*] Read chunks: {list(self.greyscale_chunks.keys())}")


    # reassamble and save the edited png based on dictionary of chunks after greyscaling
    def reassemble_png(self):
        print(f"[*] Reassembling edited png as: {self.target}")

        png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        with open(self.target, "wb") as f:
            f.write(png_signature)
            f.write(self.greyscale_chunks[b"IHDR"])
            f.write(self.greyscale_chunks[b"IDAT"])
            f.write(self.chunks[b"eXIf"])
            f.write(self.chunks[b"hASh"])
            f.write(self.chunks[b"sIGn"])
            f.write(self.greyscale_chunks[b"IEND"])

        print(f"[*] Saved greyscaled image with preserved custom chunks to {self.target}")


# main function
def main():
    # editor functionality
    editor = Editor("original.png")
    editor.parse_png()
    editor.greyscale()
    editor.reassemble_png()


if __name__ == '__main__':
    main()