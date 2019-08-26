//
// Created by Ilia.Motornyi on 26/08/2019.
//
#include "main.h"

extern TIM_HandleTypeDef HMOTOR_TIM;

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


