#ifndef HACKATHON19_DESKOVERY_RUST_HEADER_H
#define HACKATHON19_DESKOVERY_RUST_HEADER_H

#include <stdbool.h>

bool deskoveryMotor(int pwrLeft,int pwrRight, bool recovery);

typedef struct {
    volatile int alarmRatio10 ;
    volatile bool alarms[4];
    volatile bool alarm;
    volatile int darkResponse[4];
    volatile int lightResponse[4];
} PrxData;

extern volatile PrxData prxData;

extern volatile long left_ticks;//todo fn
extern volatile long right_ticks;//todo fn
extern volatile int radar_range;//todo fn

unsigned long system_ticks();
void led_control(bool on);

void display_bg_control(int brightness);

void LCD5110_write_char(unsigned char c);

void LCD5110_clear(void);

void LCD5110_set_XY(unsigned char X,unsigned char Y);

void LCD5110_write_string(const unsigned char *s);

void LCD5110_write_pict(const unsigned char *p);

void delay_ms(long ms);

#endif //HACKATHON19_DESKOVERY_RUST_HEADER_H
