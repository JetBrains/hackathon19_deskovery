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
    __HAL_UART_ENABLE_IT(&huart3, UART_IT_RXNE);
    __HAL_UART_ENABLE_IT(&huart3, UART_IT_ERR);
}

__unused void uart_output(const char *p, const int len) {
    HAL_UART_Transmit(&huart3, (uint8_t *) p, len, 1000);
}

__unused int uart_input(char *p, const int maxLen) {
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

