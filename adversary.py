import struct
import zlib
import ecdsa
from PIL import Image


# class for PNG parsing and I/O
class PNGUtils:
    def __init__(self, image):
        self.image = image
        self.png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        # {chunk_type : (length, chunk_type, data, crc)}
        self.chunks = {} 

    # read all chunks from the PNG
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

        print(f"[*] Read chunks: {list(self.chunks.keys())}")


# class for the adversary functionality 
class Adversary:
    def __init__(self, image):
        self.image = image
        self.png_utils = PNGUtils(self.image)

    
    # construct a chunk for embedding in the target image
    def construct_chunk(self, chunk_type, chunk_data):
        print(f"[*] Constructing a {chunk_type} chunk")

        chunk_length = struct.pack(">I", len(chunk_data))
        chunk_crc = struct.pack(">I", zlib.crc32(chunk_type + chunk_data) & 0xffffffff)

        return chunk_length + chunk_type + chunk_data + chunk_crc


    # reconstruct the image given the edited chunks
    def reconstruct_image(self, chunks, new_image):
        print(f"[*] Reconstructing chunks into: {new_image}")

        with open(new_image, "wb") as f:
            f.write(self.png_signature)
            for _, chunk_info in chunks.items():
                length, type, data, _ = chunk_info
                new_crc = zlib.crc32(type + data) & 0xffffffff
                f.write(struct.pack(">I", length))
                f.write(type)
                f.write(data)
                f.write(struct.pack(">I", new_crc))


    # signature swapping attack
    def signature_swap(self):
        print("[*] Resigning PNG using non-trusted device signing key")


# main function
def main():
    pass


if __name__ == '__main__':
    main()