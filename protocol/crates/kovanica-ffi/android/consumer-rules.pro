# Keep JNA and UniFFI classes
-keep class com.sun.jna.** { *; }
-keep class uniffi.kovanica.** { *; }

# Keep Kotlin metadata
-keep class kotlin.Metadata { *; }

# Don't warn about JNA
-dontwarn com.sun.jna.**
-dontwarn uniffi.kovanica.**
