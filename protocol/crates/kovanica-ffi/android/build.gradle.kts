plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "uniffi.kovanica"
    compileSdk = 34

    defaultConfig {
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "0.2.0"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildFeatures {
        aidl = true
    }

    packagingOptions {
        jniLibs {
            pickFirsts += "libkovanica_ffi.so"
        }
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
}
