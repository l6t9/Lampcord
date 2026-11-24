# JNI entry points are resolved by their class and method names.
-keep class me.lampu.lampcord.shared.voice.NativeVoice { *; }

-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { *; }
-keep class * implements com.sun.jna.Callback { *; }
-keep class uniffi.** { *; }
-dontwarn com.sun.jna.**
-dontwarn java.awt.**
