using LinearAlgebra

# Although I am looking at the code in DaltonLens-Python,
# My understanding of this code comes from this page:
# https://daltonlens.org/understanding-cvd-simulation/
# Accessed 2024-04-11.

## This section is based on https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/convert.py.

# https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/convert.py#L186
XYZJuddVos_from_linearRGB_BT709 = 1e-2 .* [
    40.9568 35.5041 17.9167;
    21.3389 70.6743 7.98680;
    1.86297 11.4620 91.2367;
]

# https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/convert.py#L160
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

# Currently unused
begin
    # Same as in color.rs
    Oklab_1_from_linearRGB = [
        0.4122214708 0.5363325363 0.0514459929;
        0.2119034982 0.6806995451 0.1073969566;
        0.0883024619 0.2817188376 0.6299787005;
    ]

    Oklab_1_from_LMS = Oklab_1_from_linearRGB * linearRGB_from_LMS

    println(Oklab_1_from_LMS)
end

## This section is based on https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/simulate.py.

# white = LMS_from_XYZ * [0.8, 0.8, 0.8]

# Using sRGB white. Mirror of https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/simulate.py#L253
white = LMS_from_linearRGB * [1, 1, 1]

# https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/simulate.py#L235
lms_475 = LMS_from_XYZ * [0.13287, 0.11284, 0.9422 ]
lms_575 = LMS_from_XYZ * [0.84394, 0.91558, 0.00197]

lms_485 = LMS_from_XYZ * [0.05699, 0.16987, 0.5864 ]
lms_660 = LMS_from_XYZ * [0.16161, 0.061  , 0.00001]

# https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/simulate.py#L127
function lms_confusion_axis(i)
    out = zeros(Float64, 3)
    out[i] = 1.0
    out
end

# https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/simulate.py#L95
function projection_matrix(n, i)
    if i == 1 
        return [
            0 -n[2]/n[1] -n[3]/n[1];
            0 1 0;
            0 0 1;
        ]
    elseif i == 2
        return [
            1 0 0;
            -n[1]/n[2] 0 -n[3]/n[2];
            0 0 1;
        ]
    elseif i == 3
        return [
            1 0 0;
            0 1 0;
            -n[1]/n[3] -n[2]/n[3] 0;
        ]
    end
end

# https://github.com/DaltonLens/DaltonLens-Python/blob/3cba5e6a7c8f0e8199c8f83f1afb58eb6dab7a3d/daltonlens/simulate.py#L261
function compute_matrices(lms_on_wing1, lms_on_wing2, anomaly)

    n1 = cross(white, lms_on_wing1) # first plane
    n2 = cross(white, lms_on_wing2) # second plane
    n_sep_plane = cross(white, lms_confusion_axis(anomaly)) # separation plane going through the diagonal
    # Swap the input so that wing1 is on the positive side of the separation plane
    if dot(n_sep_plane, lms_on_wing1) < 0
        # print("Swapped!") # I was curious. It happens for protan and tritan, not deutan
        # tweak: instead of swapping the inputs, just negate the separation plane.
        n_sep_plane .= .- n_sep_plane
    end

    H1 = projection_matrix(n1, anomaly)
    H2 = projection_matrix(n2, anomaly)

    return (n_sep_plane, H1, H2)
end

protan = compute_matrices(lms_475, lms_575, 1)
deutan = compute_matrices(lms_475, lms_575, 2)
tritan = compute_matrices(lms_485, lms_660, 3)

println(protan)
println(deutan)
println(tritan)