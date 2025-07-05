void _start() {
    char* str = "Hello from user program!\n";
    char* vga = (char*)0xB8000;
    while (*str) {
        *vga++ = *str++;
        *vga++ = 0x07; // white-on-black attribute
    }
}