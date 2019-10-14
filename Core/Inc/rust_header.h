#ifndef HACKATHON19_DESKOVERY_RUST_HEADER_H
#define HACKATHON19_DESKOVERY_RUST_HEADER_H

#include <stdbool.h>

bool delegate_deskovery_motor(int pwrLeft, int pwrRight, bool recovery);

typedef struct {
    volatile int alarmRatio10 ;
    volatile bool alarms[4];
    volatile bool alarm;
    volatile int darkResponse[4];
    volatile int lightResponse[4];
} PrxData;

extern volatile PrxData prxData;

long delegate_left_ticks();
long delegate_right_ticks();
int delegate_radar_range();

unsigned long delegate_system_ticks();
void delegate_led_control(bool on);
//todo make as u16
#define BLACK       0x0000
#define NAVY        0x000F
#define DARKGREEN   0x03E0
#define DARKCYAN    0x03EF
#define MAROON      0x7800
#define PURPLE      0x780F
#define OLIVE       0x7BE0
#define LIGHTGREY   0xC618
#define DARKGREY    0x7BEF
#define BLUE        0x001F
#define GREEN       0x07E0
#define CYAN        0x07FF
#define RED         0xF800
#define MAGENTA     0xF81F
#define YELLOW      0xFFE0
#define WHITE       0xFFFF
#define ORANGE      0xFD20
#define GREENYELLOW 0xAFE5
#define PINK        0xF81F

#define SCREEN_VERTICAL_1		0
#define SCREEN_HORIZONTAL_1		1
#define SCREEN_VERTICAL_2		2
#define SCREEN_HORIZONTAL_2		3

void delegate_display_bg_control(int brightness);

void ILI9341_Fill_Screen(unsigned short Colour);
void ILI9341_Draw_Colour(unsigned short Colour);
void ILI9341_Draw_Pixel(unsigned short X,unsigned short Y,unsigned short Colour);


void ILI9341_Draw_Rectangle(unsigned short X, unsigned short Y, unsigned short Width, unsigned short Height, unsigned short Colour);
void ILI9341_Draw_Horizontal_Line(unsigned short X, unsigned short Y, unsigned short Width, unsigned short Colour);
void ILI9341_Draw_Vertical_Line(unsigned short X, unsigned short Y, unsigned short Height, unsigned short Colour);

void ILI9341_Draw_Hollow_Circle(unsigned short X, unsigned short Y, unsigned short Radius, unsigned short Colour);
void ILI9341_Draw_Filled_Circle(unsigned short X, unsigned short Y, unsigned short Radius, unsigned short Colour);
void ILI9341_Draw_Hollow_Rectangle_Coord(unsigned short X0, unsigned short Y0, unsigned short X1, unsigned short Y1, unsigned short Colour);
void ILI9341_Draw_Filled_Rectangle_Coord(unsigned short X0, unsigned short Y0, unsigned short X1, unsigned short Y1, unsigned short Colour);
void ILI9341_Draw_Char(char Character, unsigned short X, unsigned short Y, unsigned short Colour, unsigned char Size, unsigned short Background_Colour);
void ILI9341_Draw_Text_Len(const char* Text, unsigned char len, unsigned short X, unsigned short Y, unsigned short Colour,unsigned char Size, unsigned short Background_Colour);
void ILI9341_Draw_Filled_Rectangle_Size_Text(unsigned short X0, unsigned short Y0, unsigned short Size_X, unsigned short Size_Y, unsigned short Colour);

//todo make as u8
#define HORIZONTAL_IMAGE	0
#define VERTICAL_IMAGE		1
extern const char ferris[320*240*2];
extern const char jb_logo[320*240*2];
extern const char cl_logo[320*240*2];

//USING CONVERTER: http://www.digole.com/tools/PicturetoC_Hex_converter.php
//65K colour (2Bytes / Pixel)
void ILI9341_Draw_Image(const char* Image_Array, unsigned char Orientation);

void delegate_delay_ms(long ms);

void debug_output(const unsigned char *p, unsigned int len);

void uart_output(const unsigned char *p, unsigned int len);
int  uart_input(unsigned char *p, unsigned int maxLen);

void delegate_idle();

void Error_Handler(void);

#define PRX_FL 3
#define PRX_FR 1
#define PRX_BL 0
#define PRX_BR 2


#endif //HACKATHON19_DESKOVERY_RUST_HEADER_H
