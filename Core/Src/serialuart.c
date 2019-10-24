//
// Created by Ilia.Motornyi on 19/09/2019.
//
#include "main.h"

extern UART_HandleTypeDef huart3;

#define BUF_SIZE 2000

static volatile char rcvBuff[BUF_SIZE];
static volatile unsigned int wrIdx;
static volatile unsigned int rdIdx;

static inline unsigned int incIndex(volatile unsigned int *idx) {
    *idx = (1 + *idx) % BUF_SIZE;
    return *idx;
}

void setupWifi() {
    HAL_GPIO_WritePin(ESP8266_EN_GPIO_Port, ESP8266_EN_Pin, GPIO_PIN_SET);
    HAL_GPIO_WritePin(ESP8266_RST_GPIO_Port, ESP8266_RST_Pin, GPIO_PIN_SET);
    HAL_Delay(300);
    uint8_t c[3];
    HAL_UART_Receive(&huart3, c, 3, 2);
    __HAL_UART_ENABLE_IT(&huart3, UART_IT_RXNE);
    __HAL_UART_ENABLE_IT(&huart3, UART_IT_ERR);
}

__unused void uart_output(const unsigned char *p, const unsigned int len) {
    HAL_UART_Transmit(&huart3, (uint8_t *) p, len, 1000);
}

__unused int uart_input(unsigned char *p, const unsigned int maxLen) {
    __disable_irq();
    int i = 0;
    for (; (rdIdx != wrIdx) && (i < maxLen); i++) {
        p[i] = rcvBuff[rdIdx];
        incIndex(&rdIdx);
        __enable_irq();
        __NOP();
        __disable_irq();
    }
    __enable_irq();
    return i;
}

void wifiIrqHandler() {
    uint8_t b = huart3.Instance->RDR & 0xffu;
    rcvBuff[wrIdx] = b;
    if (incIndex(&wrIdx) == rdIdx) {
        incIndex(&rdIdx);
    }
    __HAL_UART_CLEAR_IT(&huart3, UART_CLEAR_OREF | UART_CLEAR_NEF | UART_CLEAR_PEF | UART_CLEAR_FEF);
    __HAL_UART_CLEAR_FLAG(&huart3, UART_CLEAR_OREF | UART_CLEAR_NEF | UART_CLEAR_PEF | UART_CLEAR_FEF);
}

