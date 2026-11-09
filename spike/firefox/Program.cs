using System.Diagnostics;
using System.Text;

namespace WhatsappRsFirefoxDeneme;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        ApplicationConfiguration.Initialize();
        string root = AppContext.BaseDirectory;
        string browser = Path.Combine(root, "firefox", "firefox.exe");
        string profile = Path.Combine(root, "data", "profil");
        try
        {
            if (!File.Exists(browser)) throw new FileNotFoundException("Firefox motoru bulunamadı. ZIP içindeki tüm dosyaları bir klasöre çıkardığından emin ol.");
            Directory.CreateDirectory(Path.Combine(profile, "chrome"));
            string prefs = Path.Combine(profile, "user.js");
            if (!File.Exists(prefs)) File.WriteAllText(prefs, "user_pref(\"toolkit.legacyUserProfileCustomizations.stylesheets\", true);\nuser_pref(\"browser.tabs.drawInTitlebar\", false);\nuser_pref(\"browser.shell.checkDefaultBrowser\", false);\nuser_pref(\"browser.startup.homepage_override.mstone\", \"ignore\");\n", new UTF8Encoding(false));
            File.WriteAllText(Path.Combine(profile, "chrome", "userChrome.css"), "#TabsToolbar, #nav-bar, #PersonalToolbar { visibility: collapse !important; }\n", new UTF8Encoding(false));
            var start = new ProcessStartInfo(browser) { UseShellExecute = false, WorkingDirectory = Path.GetDirectoryName(browser)! };
            start.ArgumentList.Add("-new-instance");
            start.ArgumentList.Add("-profile");
            start.ArgumentList.Add(profile);
            start.ArgumentList.Add("-new-window");
            start.ArgumentList.Add("https://web.whatsapp.com/");
            Process.Start(start);
        }
        catch (Exception ex)
        {
            MessageBox.Show("Firefox denemesi açılamadı:\n\n" + ex.Message, "WhatsappRS Firefox Deneme", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }
}

