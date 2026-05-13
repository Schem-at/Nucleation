package com.github.schemat.nucleation;

import java.io.IOException;
import java.io.InputStream;
import java.net.JarURLConnection;
import java.net.URL;
import java.net.URLConnection;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.util.Enumeration;
import java.util.Locale;
import java.util.TreeSet;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;

/**
 * Resolves the appropriate platform-specific cdylib at runtime and
 * loads it via {@link System#load(String)}.
 *
 * <p>The fat JAR ships every supported {@code (os, arch)} variant under
 * {@code /native/<os>-<arch>/}. On first load we:
 * <ol>
 *   <li>Detect the host platform.</li>
 *   <li>Copy the matching cdylib to a per-version temp directory keyed by a
 *       SHA-256 of its content so multiple consumers (and multiple Nucleation
 *       versions) coexist without conflict.</li>
 *   <li>Call {@code System.load} on the extracted absolute path.</li>
 * </ol>
 *
 * <p>Behaviour can be overridden by system properties:
 * <ul>
 *   <li>{@code nucleation.native.path} — absolute path to a cdylib to load
 *       directly (bypasses extraction). Useful for development.</li>
 *   <li>{@code nucleation.native.dir} — directory to scan for a
 *       platform-named subfolder before falling back to JAR resources.</li>
 *   <li>{@code nucleation.native.debug} — when "true", prints loader steps to
 *       stderr.</li>
 * </ul>
 */
final class NativeLoader {

    private static final Object LOCK = new Object();
    private static volatile boolean LOADED = false;
    private static final boolean DEBUG = Boolean.parseBoolean(
            System.getProperty("nucleation.native.debug", "false"));

    private NativeLoader() {}

    static void loadOnce() {
        if (LOADED) return;
        synchronized (LOCK) {
            if (LOADED) return;
            doLoad();
            LOADED = true;
        }
    }

    private static void doLoad() {
        String override = System.getProperty("nucleation.native.path");
        if (override != null && !override.isBlank()) {
            log("loading override path: " + override);
            System.load(override);
            return;
        }

        String osName = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        String osArch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
        String platform = detectPlatform(osName, osArch);
        String libFile = libFileFor(osName);
        String resourcePath = "/native/" + platform + "/" + libFile;

        log("platform=" + platform + " libFile=" + libFile);

        String externalDir = System.getProperty("nucleation.native.dir");
        if (externalDir != null && !externalDir.isBlank()) {
            Path candidate = Path.of(externalDir, platform, libFile);
            if (Files.exists(candidate)) {
                log("loading from external dir: " + candidate);
                System.load(candidate.toAbsolutePath().toString());
                return;
            }
        }

        URL resource = NativeLoader.class.getResource(resourcePath);
        if (resource == null) {
            String bundled = String.join(", ", listBundledPlatforms());
            throw new UnsatisfiedLinkError(
                    "Nucleation native library is not bundled for platform " + platform + ".\n"
                            + "  expected JAR resource : " + resourcePath + "\n"
                            + "  bundled in this JAR   : " + (bundled.isEmpty() ? "<none>" : bundled) + "\n"
                            + "  host                  : os.name=" + osName + " os.arch=" + osArch + "\n"
                            + "Fix options:\n"
                            + "  - Use a multi-platform JAR from CI (GitHub Releases).\n"
                            + "  - Locally rebuild with all platforms:\n"
                            + "      ./nucleation-jvm/build-cross.sh && (cd nucleation-jvm/jvm && ./gradlew jar)\n"
                            + "  - Or build just the missing target:\n"
                            + "      ./nucleation-jvm/build-cross.sh " + platform + "\n"
                            + "  - Or point at an external cdylib at runtime:\n"
                            + "      -Dnucleation.native.path=/abs/path/to/" + libFile + "\n"
                            + "      -Dnucleation.native.dir=/abs/path/with/<platform>/<lib> subdirs");
        }

        try (InputStream in = resource.openStream()) {
            byte[] bytes = in.readAllBytes();
            String hash = sha256Short(bytes);
            String version = readVersionFromManifest();
            Path baseDir = tempBase(version, hash);
            Files.createDirectories(baseDir);
            Path target = baseDir.resolve(libFile);

            if (!Files.exists(target) || Files.size(target) != bytes.length) {
                Path tmp = Files.createTempFile(baseDir, "extract-", ".tmp");
                Files.write(tmp, bytes);
                Files.move(tmp, target, StandardCopyOption.REPLACE_EXISTING,
                        StandardCopyOption.ATOMIC_MOVE);
            }

            log("loading extracted library: " + target);
            System.load(target.toAbsolutePath().toString());
        } catch (IOException e) {
            throw new UnsatisfiedLinkError(
                    "Failed to extract Nucleation native library: " + e.getMessage());
        }
    }

    private static String detectPlatform(String osName, String osArch) {
        boolean isArm64 = osArch.contains("aarch64") || osArch.contains("arm64");
        if (osName.contains("mac") || osName.contains("darwin")) {
            return isArm64 ? "macos-arm64" : "macos-x64";
        }
        if (osName.contains("linux")) {
            return isArm64 ? "linux-arm64" : "linux-x64";
        }
        if (osName.contains("windows")) {
            return "windows-x64";
        }
        throw new UnsatisfiedLinkError("Unsupported OS: " + osName + " / " + osArch);
    }

    private static String libFileFor(String osName) {
        if (osName.contains("mac") || osName.contains("darwin")) return "libnucleation_jvm.dylib";
        if (osName.contains("windows")) return "nucleation_jvm.dll";
        return "libnucleation_jvm.so";
    }

    private static String sha256Short(byte[] data) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] hash = md.digest(data);
            StringBuilder sb = new StringBuilder(16);
            for (int i = 0; i < 8; i++) {
                sb.append(String.format("%02x", hash[i]));
            }
            return sb.toString();
        } catch (Exception e) {
            return Long.toHexString(System.nanoTime());
        }
    }

    private static Path tempBase(String version, String hash) {
        String user = System.getProperty("user.name", "anon").replaceAll("[^a-zA-Z0-9_-]", "_");
        return Path.of(System.getProperty("java.io.tmpdir"),
                "nucleation-" + version + "-" + user, hash);
    }

    private static String readVersionFromManifest() {
        Package pkg = NativeLoader.class.getPackage();
        if (pkg != null) {
            String v = pkg.getImplementationVersion();
            if (v != null && !v.isBlank()) return v;
        }
        return "dev";
    }

    private static void log(String msg) {
        if (DEBUG) System.err.println("[nucleation-loader] " + msg);
    }

    /**
     * Best-effort introspection of every {@code native/<platform>/} directory
     * present in the JAR holding this class. Used by the error path to give
     * operators a clear picture of what's actually bundled. Returns an empty
     * set if the class wasn't loaded from a JAR or scanning fails.
     */
    private static TreeSet<String> listBundledPlatforms() {
        TreeSet<String> found = new TreeSet<>();
        try {
            URL self = NativeLoader.class.getResource("NativeLoader.class");
            if (self == null) return found;
            URLConnection conn = self.openConnection();
            if (!(conn instanceof JarURLConnection)) return found;
            try (JarFile jar = ((JarURLConnection) conn).getJarFile()) {
                Enumeration<JarEntry> entries = jar.entries();
                while (entries.hasMoreElements()) {
                    String name = entries.nextElement().getName();
                    if (!name.startsWith("native/")) continue;
                    int slash = name.indexOf('/', "native/".length());
                    if (slash > 0) {
                        found.add(name.substring("native/".length(), slash));
                    }
                }
            }
        } catch (IOException ignored) {
            // Falls back to empty set — error message will say "<none>".
        }
        return found;
    }
}
