plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "io.kovanica.wallet"
    compileSdk = 34

    defaultConfig {
        applicationId = "io.kovanica.wallet"
        minSdk = 24
        targetSdk = 34
        versionCode = 3
        versionName = "0.2.1"
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
    implementation(files("libs/core-3.5.4.jar"))
}
