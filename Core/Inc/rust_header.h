#ifndef HACKATHON19_DESKOVERY_RUST_HEADER_H
#define HACKATHON19_DESKOVERY_RUST_HEADER_H

#include <stdbool.h>

bool deskovery_motor(int pwrLeft,int pwrRight, bool recovery);

typedef struct {
    volatile int alarmRatio10 ;
    volatile bool alarms[4];
    volatile bool alarm;
    volatile int darkResponse[4];
    volatile int lightResponse[4];
} PrxData;

extern volatile PrxData prxData;

long left_ticks();
long right_ticks();
int radar_range();

unsigned long system_ticks();
void led_control(bool on);

void display_bg_control(int brightness);

void LCD5110_write_char(unsigned char c);

void LCD5110_clear(void);

void LCD5110_set_XY(unsigned char X,unsigned char Y);

void LCD5110_write_bytes(unsigned const char *s, unsigned int len);

void LCD5110_write_pict(const unsigned char *p);

void delay_ms(long ms);

void debug_output(const unsigned char *p, unsigned int len);

void uart_output(const char *p, int len);
int  uart_input(char *p, int maxLen);

void idle();

void Error_Handler(void);

#define PRX_FL 3
#define PRX_FR 1
#define PRX_BL 0
#define PRX_BR 2


#endif //HACKATHON19_DESKOVERY_RUST_HEADER_H
