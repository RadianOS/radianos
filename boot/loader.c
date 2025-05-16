#include <stdint.h>

#define VIDEO_MEM ((volatile uint16_t*)0xB8000)
#define SCREEN_WIDTH 80
#define SCREEN_HEIGHT 25

void clear_screen(uint8_t color) {
    uint16_t blank = (color << 8) | ' ';
    for(int i = 0; i < SCREEN_WIDTH * SCREEN_HEIGHT; i++) {
        VIDEO_MEM[i] = blank;
    }
}

void print_string(int row, int col, const char* str, uint8_t color) {
    volatile uint16_t* pos = VIDEO_MEM + row * SCREEN_WIDTH + col;
    while(*str) {
        *pos++ = ((uint16_t)color << 8) | *str++;
    }
}

void draw_box(int row, int col, int width, int height, uint8_t color) {
    // Draw corners
    print_string(row, col, "+", color);
    print_string(row, col + width - 1, "+", color);
    print_string(row + height - 1, col, "+", color);
    print_string(row + height - 1, col + width - 1, "+", color);

    // Draw top/bottom
    for(int c = col + 1; c < col + width - 1; c++) {
        print_string(row, c, "-", color);
        print_string(row + height - 1, c, "-", color);
    }

    // Draw sides
    for(int r = row + 1; r < row + height -1; r++) {
        print_string(r, col, "|", color);
        print_string(r, col + width - 1, "|", color);
    }
}

const char* logo[] = {
    "   .'..",
    "   .'''''..",
    "    '''''''''..",
    "    '''''''''''''..",
    "    ''''''''''''''''''..",
    "    ''''''''''''''''''''''..",
    "    '''''''''''''''''''''''''..",
    "    '''''''''''''''''''''''''''''..",
    "    ''''''''''''''''''''''''''''''..",
    "    ''''''''''''''''''''''''''''.......",
    "    ''''''''''''''''''''''''''......",
    "    '''''''''''''''''''''''.....",
    "    '''''''''''''''''''''....",
    "    .'''''''''''''''''....       ...",
    "    .'''''''''''''''...       ....''",
    "    .''''''''''''..        ...''''''.",
    "    .''''''''''.        ..''''''''''.",
    "     '''''''.           ''''''''''''",
    "     ''''.                   '''''''",
    "     '.                           '''",
    0
};

void draw_logo(int row, int col, uint8_t color) {
    for(int i = 0; logo[i] != 0; i++) {
        print_string(row + i, col, logo[i], color);
    }
}

void print_menu(int row, int col, uint8_t color) {
    const char* items[] = {
        "Start RadianOS",
        "Options",
        "Reboot",
        0
    };
    for(int i = 0; items[i] != 0; i++) {
        print_string(row + i, col, items[i], color);
    }
}

void main() {
    clear_screen(0x1F); // blue background white text
    draw_box(8, 4, 40, 10, 0x1F);
    draw_logo(3, 55, 0x0F);
    print_menu(10, 6, 0x0F);

    while(1) {
        // TODO: wait for keypress and handle menu
    }
}

