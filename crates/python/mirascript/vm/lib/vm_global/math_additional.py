import math
from .._helpers import required
from ...operations import ToString_,ToNumber_
from mirascript.vm.operations import numberToString_,is_decimal_number
GAMMA_G = 4.742_187_5

GAMMA_P = [
    0.999_999_999_999_997_091_82, 57.156_235_665_862_923_517, -59.597_960_355_475_491_248, 14.136_097_974_741_747_174,
    -0.491_913_816_097_620_199_78, 0.339_946_499_848_118_886_99e-4, 0.465_236_289_270_485_756_65e-4,
    -0.983_744_753_048_795_646_77e-4, 0.158_088_703_224_912_488_84e-3, -0.210_264_441_724_104_883_19e-3,
    0.217_439_618_115_212_643_2e-3, -0.164_318_106_536_763_890_22e-3, 0.844_182_239_838_527_432_93e-4,
    -0.261_908_384_015_814_086_7e-4, 0.368_991_826_595_316_227_04e-5,
]

def factorial(x):
    required('x', x, None)
    n = ToNumber_(x)
    if math.isnan(n) or n < 0 :
        return math.nan
    if n >= 171:
        return math.inf
    if not is_decimal_number(n) :
        # return math.inf
        if n ==0 or n==1:
            return 1
        r =1.0
        for i in range(2,int(n)+1):
            
            r *= i
        return r
    if n > 85:
      n =n +1.0
      twoN = n * n
      threeN = twoN * n
      fourN = threeN * n
      fiveN = fourN * n
      x= math.sqrt(2 * math.pi / n) * math.pow(n / math.e, n) * ( 1 + 1 / (12 * n) + 1 / (288 * twoN) - 139 / (51840 * threeN) - 571 / (2488320 * fourN) + 163879 / (209018880 * fiveN) + 5246819 / (75246796800 * n * fiveN))
      return x
   
    p = GAMMA_P[0]
    for k in range(1, len(GAMMA_P)):
        p += GAMMA_P[k] / (n + k)
    t = n + GAMMA_G + 0.5
    return math.sqrt(2 * math.pi) * math.pow(t, n + 0.5) * math.exp(-t) * p
    