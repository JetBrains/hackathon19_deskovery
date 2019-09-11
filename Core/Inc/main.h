/* USER CODE BEGIN Header */
/**
  ******************************************************************************
  * @file           : main.h
  * @brief          : Header for main.c file.
  *                   This file contains the common defines of the application.
  ******************************************************************************
  * @attention
  *
  * <h2><center>&copy; Copyright (c) 2019 STMicroelectronics.
  * All rights reserved.</center></h2>
  *
  * This software component is licensed by ST under BSD 3-Clause license,
  * the "License"; You may not use this file except in compliance with the
  * License. You may obtain a copy of the License at:
  *                        opensource.org/licenses/BSD-3-Clause
  *
  ******************************************************************************
  */
/* USER CODE END Header */

/* Define to prevent recursive inclusion -------------------------------------*/
#ifndef __MAIN_H
#define __MAIN_H

#ifdef __cplusplus
extern "C" {
#endif

/* Includes ------------------------------------------------------------------*/
#include "stm32l4xx_hal.h"

/* Private includes ----------------------------------------------------------*/
/* USER CODE BEGIN Includes */
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include "vl53l1_api.h"
/* USER CODE END Includes */

/* Exported types ------------------------------------------------------------*/
/* USER CODE BEGIN ET */

/* USER CODE END ET */

/* Exported constants --------------------------------------------------------*/
/* USER CODE BEGIN EC */

/* USER CODE END EC */

/* Exported macro ------------------------------------------------------------*/
/* USER CODE BEGIN EM */

/* USER CODE END EM */

void HAL_TIM_MspPostInit(TIM_HandleTypeDef *htim);

/* Exported functions prototypes ---------------------------------------------*/
void Error_Handler(void);

/* USER CODE BEGIN EFP */
void deskoveryInit(void);
void deskoveryReadEncoders(void);
bool deskoveryMotor(int pwrLeft,int pwrRight, bool recovery);
/* USER CODE END EFP */

/* Private defines -----------------------------------------------------------*/
#define B1_Pin GPIO_PIN_13
#define B1_GPIO_Port GPIOC
#define PRX_LF_Pin GPIO_PIN_0
#define PRX_LF_GPIO_Port GPIOC
#define PRX_RF_Pin GPIO_PIN_1
#define PRX_RF_GPIO_Port GPIOC
#define PRX_LB_Pin GPIO_PIN_2
#define PRX_LB_GPIO_Port GPIOC
#define PRX_RB_Pin GPIO_PIN_3
#define PRX_RB_GPIO_Port GPIOC
#define ENC_L1_Pin GPIO_PIN_0
#define ENC_L1_GPIO_Port GPIOA
#define ENC_L2_Pin GPIO_PIN_1
#define ENC_L2_GPIO_Port GPIOA
#define USART_TX_Pin GPIO_PIN_2
#define USART_TX_GPIO_Port GPIOA
#define USART_RX_Pin GPIO_PIN_3
#define USART_RX_GPIO_Port GPIOA
#define LD2_Pin GPIO_PIN_5
#define LD2_GPIO_Port GPIOA
#define PRX_EN_Pin GPIO_PIN_0
#define PRX_EN_GPIO_Port GPIOB
#define MR_DIR_Pin GPIO_PIN_1
#define MR_DIR_GPIO_Port GPIOB
#define ML_DIR_Pin GPIO_PIN_13
#define ML_DIR_GPIO_Port GPIOB
#define ML_PWM_Pin GPIO_PIN_14
#define ML_PWM_GPIO_Port GPIOB
#define MR_PWM_Pin GPIO_PIN_15
#define MR_PWM_GPIO_Port GPIOB
#define ENC_R1_Pin GPIO_PIN_6
#define ENC_R1_GPIO_Port GPIOC
#define ENC_R2_Pin GPIO_PIN_7
#define ENC_R2_GPIO_Port GPIOC
#define VL53_RST_Pin GPIO_PIN_8
#define VL53_RST_GPIO_Port GPIOC
#define TMS_Pin GPIO_PIN_13
#define TMS_GPIO_Port GPIOA
#define TCK_Pin GPIO_PIN_14
#define TCK_GPIO_Port GPIOA
#define SWO_Pin GPIO_PIN_3
#define SWO_GPIO_Port GPIOB
/* USER CODE BEGIN Private defines */

#define HMOTOR_TIM htim15

#define ML_TIM_CH TIM_CHANNEL_1
#define MR_TIM_CH TIM_CHANNEL_2

#define EL_TIM htim5
#define ER_TIM htim8

extern volatile long left_ticks;
extern volatile long right_ticks;

/* USER CODE END Private defines */

#ifdef __cplusplus
}
#endif

#endif /* __MAIN_H */

/************************ (C) COPYRIGHT STMicroelectronics *****END OF FILE****/
