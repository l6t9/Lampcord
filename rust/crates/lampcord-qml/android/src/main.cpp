// Qt's Android template calls this native main() after loading the app
// library. The real application is the Rust entry point exported by the
// lampcord-qml static library.
extern "C" int lampcord_qml_main();

int main(int argc, char **argv)
{
    (void)argc;
    (void)argv;
    return lampcord_qml_main();
}
