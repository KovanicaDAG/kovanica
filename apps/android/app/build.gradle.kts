plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "io.kovanica.light"
    compileSdk = 34

    defaultConfig {
        applicationId = "io.kovanica.light"
        minSdk = 24
        targetSdk = 34
        versionCode = 11
        versionName = "0.8.1"
        ndk {
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    buildFeatures {
        // Settings reads BuildConfig.VERSION_NAME / VERSION_CODE for the About card.
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    // Vendored zxing core only (encode + decode, plain Java, no androidx). See libs/.
    implementation(files("libs/core-3.5.4.jar"))
}
