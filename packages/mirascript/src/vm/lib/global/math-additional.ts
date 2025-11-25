/* eslint-disable no-loss-of-precision */
import { isInteger, isNaN } from '../../../helpers/utils.js';
import { expectNumber, VmLib } from '../helpers.js';
const { sqrt, pow, exp } = Math;

const GAMMA_G = 4.742_187_5;

const GAMMA_P = [
    0.999_999_999_999_997_091_82, 57.156_235_665_862_923_517, -59.597_960_355_475_491_248, 14.136_097_974_741_747_174,
    -0.491_913_816_097_620_199_78, 0.339_946_499_848_118_886_99e-4, 0.465_236_289_270_485_756_65e-4,
    -0.983_744_753_048_795_646_77e-4, 0.158_088_703_224_912_488_84e-3, -0.210_264_441_724_104_883_19e-3,
    0.217_439_618_115_212_643_2e-3, -0.164_318_106_536_763_890_22e-3, 0.844_182_239_838_527_432_93e-4,
    -0.261_908_384_015_814_086_7e-4, 0.368_991_826_595_316_227_04e-5,
];

const SQRT_2_PI = sqrt(2 * Math.PI);

export const factorial = VmLib(
    (x): number => {
        let n = expectNumber('x', x);
        if (isNaN(n) || n < 0) return Number.NaN;
        if (n >= 171) return Number.POSITIVE_INFINITY; // will overflow

        if (isInteger(n)) {
            if (n === 0 || n === 1) return 1;

            let r = 1;
            for (let i = 2; i <= n; i++) {
                r *= i;
            }
            return r;
        }

        if (n > 85) {
            // Extended Stirling Approx
            n = n + 1;
            const twoN = n * n;
            const threeN = twoN * n;
            const fourN = threeN * n;
            const fiveN = fourN * n;
            return (
                sqrt((2 * Math.PI) / n) *
                pow(n / Math.E, n) *
                (1 +
                    1 / (12 * n) +
                    1 / (288 * twoN) -
                    139 / (51840 * threeN) -
                    571 / (2_488_320 * fourN) +
                    163_879 / (209_018_880 * fiveN) +
                    5_246_819 / (75_246_796_800 * fiveN * n))
            );
        }

        let p = GAMMA_P[0]!;
        for (let i = 1; i < GAMMA_P.length; ++i) {
            p += GAMMA_P[i]! / (n + i);
        }

        const t = n + GAMMA_G + 0.5;
        return SQRT_2_PI * pow(t, n + 0.5) * exp(-t) * p;
    },
    {
        summary: '返回一个数的阶乘',
        params: { x: '要计算阶乘的数值' },
        paramsType: { x: 'number' },
        returnsType: 'number',
        examples: ['factorial(5) // 120'],
    },
);
