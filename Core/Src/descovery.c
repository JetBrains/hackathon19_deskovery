//
// Created by Ilia.Motornyi on 26/08/2019.
//
#include "main.h"

extern TIM_HandleTypeDef HMOTOR_TIM;
extern TIM_HandleTypeDef EL_TIM;
extern TIM_HandleTypeDef ER_TIM;
extern ADC_HandleTypeDef hadc1;
volatile long left_ticks = 0;
volatile long right_ticks = 0;

extern UART_HandleTypeDef huart2;


static inline void runMotorChannel(int pwr, GPIO_TypeDef *dirPort, uint16_t dirPin, uint16_t motorChannel) {
    if (pwr >= 0) {
        HAL_GPIO_WritePin(dirPort, dirPin, GPIO_PIN_RESET);
        __HAL_TIM_SET_COMPARE(&HMOTOR_TIM, motorChannel, pwr);
    } else {
        HAL_GPIO_WritePin(dirPort, dirPin, GPIO_PIN_SET);
        __HAL_TIM_SET_COMPARE(&HMOTOR_TIM, motorChannel, -pwr);
    }
    HAL_TIM_PWM_Start(&HMOTOR_TIM, motorChannel);
}

bool deskoveryMotor(int pwrLeft, int pwrRight, bool recovery) {
//    if(! recovery && prxData.alarm) {//todo recovery
//        pwrLeft = 0;
//        pwrRight = 0;
//    }
    runMotorChannel(pwrLeft, ML_DIR_GPIO_Port, ML_DIR_Pin, ML_TIM_CH);
    runMotorChannel(pwrRight, MR_DIR_GPIO_Port, MR_DIR_Pin, MR_TIM_CH);
    return true;
}


void deskoveryInit(void) {
    HAL_TIM_Encoder_Start(&EL_TIM, TIM_CHANNEL_ALL);
    HAL_TIM_Encoder_Start(&ER_TIM, TIM_CHANNEL_ALL);
}

void deskoveryReadEncoders() {
    left_ticks += (int16_t) __HAL_TIM_GET_COUNTER(&EL_TIM);
    __HAL_TIM_SET_COUNTER(&EL_TIM, 0);
    __HAL_TIM_SET_COMPARE(&EL_TIM, TIM_CHANNEL_1, 0);
    __HAL_TIM_SET_COMPARE(&EL_TIM, TIM_CHANNEL_2, 0);
    right_ticks += (int16_t) __HAL_TIM_GET_COUNTER(&ER_TIM);
    __HAL_TIM_SET_COUNTER(&ER_TIM, 0);
    __HAL_TIM_SET_COMPARE(&ER_TIM, TIM_CHANNEL_1, 0);
    __HAL_TIM_SET_COMPARE(&ER_TIM, TIM_CHANNEL_2, 0);
}

__unused int _write(int file, char *ptr, int len) {
    HAL_UART_Transmit(&huart2, ptr, len, 200);
}

static bool prxLedOn = false;
volatile PrxData prxData = {.alarmRatio10 = 30, .alarms = {false, false, false, false}, .alarm = false,
        .darkResponse = {0, 0, 0, 0}, .lightResponse ={0, 0, 0, 0}};

void HAL_ADCEx_InjectedConvCpltCallback(__unused ADC_HandleTypeDef *hadc) {
    if (prxLedOn) {
        prxData.lightResponse[0] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_1);
        prxData.lightResponse[1] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_2);
        prxData.lightResponse[2] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_3);
        prxData.lightResponse[3] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_4);
        prxData.alarms[0] = (prxData.darkResponse[0] * 10 / (prxData.lightResponse[0] + 1)) < prxData.alarmRatio10;
        prxData.alarms[1] = (prxData.darkResponse[1] * 10 / (prxData.lightResponse[1] + 1)) < prxData.alarmRatio10;
        prxData.alarms[2] = (prxData.darkResponse[2] * 10 / (prxData.lightResponse[2] + 1)) < prxData.alarmRatio10;
        prxData.alarms[3] = (prxData.darkResponse[3] * 10 / (prxData.lightResponse[3] + 1)) < prxData.alarmRatio10;
        prxData.alarm = prxData.alarms[0] || prxData.alarms[1] || prxData.alarms[2] || prxData.alarms[3];
    } else {
        prxData.darkResponse[0] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_1);
        prxData.darkResponse[1] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_2);
        prxData.darkResponse[2] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_3);
        prxData.darkResponse[3] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_4);
    }
    if(prxData.alarm) {
        HAL_TIM_PWM_Stop(&HMOTOR_TIM,TIM_CHANNEL_1);
        HAL_TIM_PWM_Stop(&HMOTOR_TIM,TIM_CHANNEL_2);
    } else {
        HAL_TIM_PWM_Start(&HMOTOR_TIM,TIM_CHANNEL_1);
        HAL_TIM_PWM_Start(&HMOTOR_TIM,TIM_CHANNEL_2);

    }

    prxLedOn = !prxLedOn;
    HAL_GPIO_WritePin(PRX_EN_GPIO_Port, PRX_EN_Pin, prxLedOn ? GPIO_PIN_SET : GPIO_PIN_RESET);
    HAL_ADCEx_InjectedStart_IT(&hadc1);
}

