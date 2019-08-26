//
// Created by Ilia.Motornyi on 26/08/2019.
//
#include "main.h"

extern TIM_HandleTypeDef HMOTOR_TIM;
extern TIM_HandleTypeDef EL_TIM;
extern TIM_HandleTypeDef ER_TIM;
volatile long left_ticks = 0;
volatile long right_ticks = 0;

static inline void runMotorChannel(int pwr, GPIO_TypeDef * dirPort, uint16_t dirPin, uint16_t motorChannel) {
    if(pwr >= 0) {
        HAL_GPIO_WritePin(dirPort,dirPin,GPIO_PIN_RESET);
        __HAL_TIM_SET_COMPARE(&HMOTOR_TIM,motorChannel,pwr);
    } else {
        HAL_GPIO_WritePin(dirPort,dirPin,GPIO_PIN_SET);
        __HAL_TIM_SET_COMPARE(&HMOTOR_TIM,motorChannel,-pwr);
    }
    HAL_TIM_PWM_Start(&HMOTOR_TIM,motorChannel);
}
bool deskoveryMotor(int pwrLeft,int pwrRight, bool recovery) {
    //todo recovery mode
    runMotorChannel(pwrLeft,ML_DIR_GPIO_Port,ML_DIR_Pin,ML_TIM_CH);
    runMotorChannel(pwrRight,MR_DIR_GPIO_Port,MR_DIR_Pin,MR_TIM_CH);
    return true;
}


void deskoveryInit(void) {
    HAL_TIM_Encoder_Start(&EL_TIM,TIM_CHANNEL_ALL);
    HAL_TIM_Encoder_Start(&ER_TIM,TIM_CHANNEL_ALL);
}

void deskoveryReadEncoders() {
    left_ticks += (int16_t)__HAL_TIM_GET_COUNTER(&EL_TIM);
    __HAL_TIM_SET_COUNTER(&EL_TIM,0);
    __HAL_TIM_SET_COMPARE(&EL_TIM,TIM_CHANNEL_1,0);
    __HAL_TIM_SET_COMPARE(&EL_TIM,TIM_CHANNEL_2,0);
    right_ticks += (int16_t)__HAL_TIM_GET_COUNTER(&ER_TIM);
    __HAL_TIM_SET_COUNTER(&ER_TIM,0);
    __HAL_TIM_SET_COMPARE(&ER_TIM,TIM_CHANNEL_1,0);
    __HAL_TIM_SET_COMPARE(&ER_TIM,TIM_CHANNEL_2,0);
}


