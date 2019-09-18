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

extern long left_ticks();
extern long right_ticks();
extern int radar_range();//todo implement

unsigned long system_ticks();
void led_control(bool on);

void display_bg_control(int brightness);

void LCD5110_write_char(unsigned char c);

void LCD5110_clear(void);

void LCD5110_set_XY(unsigned char X,unsigned char Y);

void LCD5110_write_string(const unsigned char *s);

void LCD5110_write_pict(const unsigned char *p);

void delay_ms(long ms);

void debug_output(const unsigned char *p, unsigned int len); //todo implement

void uart_output(const unsigned char *p, int len);  //todo implement
int  uart_input(const unsigned char *p, int maxLen);  //todo implement

void idle();  //todo implement
#endif //HACKATHON19_DESKOVERY_RUST_HEADER_H
