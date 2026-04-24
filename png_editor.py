import struct
import zlib
from PIL import Image


# class for PNG parsing functionality
class PNGUtils:
    def __init__(self, image_path):
        self.image = image_path
        self.png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        # {chunk_type : (length, chunk_type, data, crc)}
        self.chunks = {} 


    # helper function to read a single chunk from the image
    def read_single_chunk(self, f):
        chunk_length, chunk_type = struct.unpack(">I4s", f.read(8))
        chunk_data = f.read(chunk_length)
        expected_crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff
        actual_crc, = struct.unpack(">I", f.read(4))

        # check the crc for data corruption
        assert expected_crc == actual_crc

        return chunk_type, chunk_data, actual_crc


    # read all chunks from the image
    def read_all_chunks(self):
        print(f"[*] Reading all chunks from: {self.image}")

        with open(self.image, "rb") as f:
            assert self.png_signature == f.read(8)

            # read chunks until we have read IEND
            while True:
                chunk_type, chunk_data, chunk_crc = self.read_single_chunk(f)
                self.chunks[chunk_type] = [len(chunk_data), chunk_type, chunk_data, chunk_crc]
                
                if chunk_type == b"IEND":
                    break


    # flip the color type field of the IHDR chunk to 0 to indicate a greyscaled image
    def flip_color_type(self):
        print("[*] Setting color type field to 0 in IHDR")
        assert b"IHDR" in self.chunks

        # the color type is the 10th byte in the IHDR chunk


    # reassemble the png given the dictionary of chunks
    def reassemble_png(self):
        print("[*] Reassembling the png from the dictionary of chunks")


# class for the editor functionality
class EditorUtils:
    def __init__(self, image_path):
        self.image = image_path
        self.target = "greyscale.png"
        # coefficients are rounded to the nearest integer to work in a circuit
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.png_utils = PNGUtils(self.image)


    # greyscale the pixels in the IDAT chunk(s) - after IDAT chunks are aggregated
    def greyscale(self):
        print(f"[*] Applying greyscale transformation to: {self.image}")


# main function
def main():
    pass


if __name__ == '__main__':
    main()