using LinearAlgebra

XYZJuddVos_from_linearRGB_BT709 = 1e-2 .* [
    40.9568 35.5041 17.9167;
    21.3389 70.6743 7.98680;
    1.86297 11.4620 91.2367;
]

LMS_from_XYZJuddVos_Smith_Pokorny_1975 = [
     0.15514 0.54312 -0.03286;
    -0.15514 0.45684  0.03286;
     0       0        0.01608;
]

XYZ_from_linearRGB = XYZJuddVos_from_linearRGB_BT709
LMS_from_XYZ = LMS_from_XYZJuddVos_Smith_Pokorny_1975

LMS_from_linearRGB = LMS_from_XYZ * XYZ_from_linearRGB

println(LMS_from_linearRGB)

linearRGB_from_LMS = inv(LMS_from_linearRGB)

print(linearRGB_from_LMS)

Oklab_1_from_linearRGB = [
    0.4122214708 0.5363325363 0.0514459929;
    0.2119034982 0.6806995451 0.1073969566;
    0.0883024619 0.2817188376 0.6299787005;
]

Oklab_1_from_LMS = Oklab_1_from_linearRGB * linearRGB_from_LMS

# Currently unused
print(Oklab_1_from_LMS)