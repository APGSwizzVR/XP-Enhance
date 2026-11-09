using System.IO;
using System.Text.Json;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Microsoft.Win32;

namespace XPEnhance;

public partial class MainWindow : Window
{
    private readonly string dataDirectory = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "XPEnhance");
    private string settingsFile = string.Empty;
    private string simulatorPath = string.Empty;
    private string profile = "Balanced";

    private sealed class VisualSetting
    {
        public double Value { get; set; }
        public bool Enabled { get; set; }

        public VisualSetting() { }

        public VisualSetting(double value, bool enabled)
        {
            Value = value;
            Enabled = enabled;
        }
    }

    private sealed class AppSettings
    {
        public string SimulatorPath { get; set; } = string.Empty;
        public string Profile { get; set; } = "Balanced";
        public Dictionary<string, VisualSetting> Visuals { get; set; } = new();
    }

    private readonly Dictionary<string, VisualSetting> visuals = new()
    {
        ["Cloud quality"] = new(80, true),
        ["Atmospheric effects"] = new(75, true),
        ["Fog realism"] = new(70, true),
        ["Sunlight"] = new(80, true),
        ["Shadow quality"] = new(75, true),
        ["Water appearance"] = new(75, true),
        ["Terrain detail"] = new(85, true),
        ["Ground detail"] = new(85, true),
        ["Colour grading"] = new(50, true),
        ["Sharpening"] = new(40, true)
    };

    public MainWindow()
    {
        InitializeComponent();
        Directory.CreateDirectory(dataDirectory);
        settingsFile = Path.Combine(dataDirectory, "settings.json");
        LoadSettings();
        ShowPage("Dashboard");
    }

    private void Nav_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button button && button.Tag is string page)
            ShowPage(page);
    }

    private void ShowPage(string page)
    {
        PageTitle.Text = page;
        PageSubtitle.Text = page switch
        {
            "Dashboard" => "Your X-Plane 12 enhancement workspace.",
            "Visual Settings" => "Adjust individual realism controls.",
            "Presets" => "Choose a starting point for your setup.",
            "Settings" => "Configure your simulator installation.",
            _ => "This module is planned for a future release."
        };

        PageContent.Content = page switch
        {
            "Dashboard" => Dashboard(),
            "Visual Settings" => VisualPage(),
            "Presets" => PresetsPage(),
            "Settings" => SettingsPage(),
            _ => ComingSoon(page)
        };
    }

    private StackPanel CreateCard()
    {
        return new StackPanel
        {
            Background = new SolidColorBrush(Color.FromRgb(25, 30, 38)),
            Margin = new Thickness(0, 0, 0, 14)
        };
    }

    private TextBlock CreateText(string text, double size = 13, bool bold = false, string color = "#F1F4F8")
    {
        return new TextBlock
        {
            Text = text,
            FontSize = size,
            FontWeight = bold ? FontWeights.SemiBold : FontWeights.Normal,
            Foreground = new SolidColorBrush((Color)ColorConverter.ConvertFromString(color)!),
            Margin = new Thickness(18, 8, 18, 8),
            TextWrapping = TextWrapping.Wrap
        };
    }

    private Button CreateButton(string text, RoutedEventHandler click)
    {
        var button = new Button
        {
            Content = text,
            HorizontalAlignment = HorizontalAlignment.Left,
            Margin = new Thickness(18, 8, 8, 12),
            Padding = new Thickness(12, 8, 12, 8)
        };
        button.Click += click;
        return button;
    }

    private UIElement Dashboard()
    {
        var root = new StackPanel();
        var welcome = CreateCard();
        welcome.Children.Add(CreateText("XP ENHANCE  —  X-PLANE 12", 11, true, "#75B8FF"));
        welcome.Children.Add(CreateText("Make your simulator yours.", 28, true));
        welcome.Children.Add(CreateText("A central place to configure and manage your X-Plane enhancement setup.", 14, false, "#9AA6B5"));
        welcome.Children.Add(CreateText("SIMULATOR INSTALLATION", 10, true, "#9AA6B5"));
        welcome.Children.Add(CreateText(string.IsNullOrWhiteSpace(simulatorPath) ? "Not configured" : simulatorPath));
        welcome.Children.Add(CreateButton("Choose X-Plane folder", (_, _) => ChooseFolder()));
        welcome.Children.Add(CreateButton("Open Visual Settings", (_, _) => ShowPage("Visual Settings")));
        root.Children.Add(welcome);

        var info = CreateCard();
        info.Children.Add(CreateText("ACTIVE PROFILE", 10, true, "#75B8FF"));
        info.Children.Add(CreateText(profile, 22, true));
        info.Children.Add(CreateText("Foundation build: settings and presets are saved locally. Rendering changes and addon installation are not implemented yet.", 13, false, "#9AA6B5"));
        root.Children.Add(info);
        return root;
    }

    private UIElement VisualPage()
    {
        var root = new StackPanel();
        var intro = CreateCard();
        intro.Children.Add(CreateText("VISUAL CONFIGURATION", 11, true, "#75B8FF"));
        intro.Children.Add(CreateText("Tune your realism profile.", 22, true));
        intro.Children.Add(CreateText("These controls currently save profile values only; they are not yet connected to X-Plane rendering.", 12, false, "#9AA6B5"));
        root.Children.Add(intro);

        foreach (var pair in visuals)
        {
            string key = pair.Key;
            VisualSetting setting = pair.Value;
            var row = new Grid { Margin = new Thickness(12, 8, 12, 8) };
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(180) });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(55) });

            var check = new CheckBox
            {
                Content = key,
                IsChecked = setting.Enabled,
                VerticalAlignment = VerticalAlignment.Center
            };
            var slider = new Slider
            {
                Minimum = 0,
                Maximum = 100,
                Value = setting.Value,
                IsEnabled = setting.Enabled,
                VerticalAlignment = VerticalAlignment.Center
            };
            var valueText = CreateText($"{setting.Value:0}%", 12, false, "#9AA6B5");
            valueText.Margin = new Thickness(4);

            check.Checked += (_, _) =>
            {
                setting.Enabled = true;
                slider.IsEnabled = true;
                SaveSettings();
            };
            check.Unchecked += (_, _) =>
            {
                setting.Enabled = false;
                slider.IsEnabled = false;
                SaveSettings();
            };
            slider.ValueChanged += (_, _) =>
            {
                setting.Value = slider.Value;
                valueText.Text = $"{slider.Value:0}%";
                SaveSettings();
            };

            Grid.SetColumn(check, 0);
            Grid.SetColumn(slider, 1);
            Grid.SetColumn(valueText, 2);
            row.Children.Add(check);
            row.Children.Add(slider);
            row.Children.Add(valueText);

            var card = CreateCard();
            card.Children.Add(row);
            root.Children.Add(card);
        }

        return root;
    }

    private UIElement PresetsPage()
    {
        var card = CreateCard();
        card.Children.Add(CreateText("PROFILE MANAGER", 11, true, "#75B8FF"));
        card.Children.Add(CreateText("Choose a starting point.", 22, true));

        var combo = new ComboBox
        {
            ItemsSource = new[] { "Balanced", "Ultra Realism", "Performance", "VR Realism", "Cinematic", "Custom" },
            SelectedItem = profile,
            Margin = new Thickness(18, 12, 18, 12),
            Padding = new Thickness(8)
        };
        card.Children.Add(combo);

        card.Children.Add(CreateButton("Apply profile", (_, _) =>
        {
            profile = combo.SelectedItem?.ToString() ?? "Balanced";
            double value = profile switch
            {
                "Ultra Realism" => 95,
                "Performance" => 45,
                "VR Realism" => 65,
                "Cinematic" => 90,
                _ => 75
            };

            foreach (var setting in visuals.Values)
                setting.Value = value;

            SaveSettings();
            FooterStatus.Text = $"Saved {profile} profile — X-Plane files unchanged";
            ShowPage("Presets");
        }));

        return card;
    }

    private UIElement SettingsPage()
    {
        var card = CreateCard();
        card.Children.Add(CreateText("SIMULATOR INSTALLATION", 11, true, "#75B8FF"));
        card.Children.Add(CreateText(string.IsNullOrWhiteSpace(simulatorPath) ? "No X-Plane folder selected" : simulatorPath));
        card.Children.Add(CreateButton("Browse for X-Plane 12", (_, _) => ChooseFolder()));
        card.Children.Add(CreateText("Settings are stored in %APPDATA%\\XPEnhance.", 12, false, "#9AA6B5"));
        return card;
    }

    private UIElement ComingSoon(string page)
    {
        var card = CreateCard();
        card.Children.Add(CreateText("PLANNED MODULE", 11, true, "#75B8FF"));
        card.Children.Add(CreateText(page, 23, true));
        card.Children.Add(CreateText("This module is on the XP Enhance roadmap and is not functional in version 0.1.0.", 13, false, "#9AA6B5"));
        return card;
    }

    private void ChooseFolder()
    {
        var dialog = new OpenFolderDialog { Title = "Select X-Plane 12 folder" };
        if (dialog.ShowDialog() != true)
            return;

        if (!File.Exists(Path.Combine(dialog.FolderName, "X-Plane.exe")) &&
            !File.Exists(Path.Combine(dialog.FolderName, "X-Plane")))
        {
            MessageBox.Show("Select the folder containing the X-Plane executable.", "Invalid folder",
                MessageBoxButton.OK, MessageBoxImage.Warning);
            return;
        }

        simulatorPath = dialog.FolderName;
        SaveSettings();
        ShowPage("Dashboard");
    }

    private void SaveSettings()
    {
        try
        {
            var settings = new AppSettings
            {
                SimulatorPath = simulatorPath,
                Profile = profile,
                Visuals = visuals
            };

            var options = new JsonSerializerOptions { WriteIndented = true };
            File.WriteAllText(settingsFile, JsonSerializer.Serialize(settings, options));
            if (FooterStatus is not null)
                FooterStatus.Text = "Ready — settings saved locally";
        }
        catch (Exception ex)
        {
            if (FooterStatus is not null)
                FooterStatus.Text = "Could not save settings: " + ex.Message;
        }
    }

    private void LoadSettings()
    {
        try
        {
            if (!File.Exists(settingsFile))
                return;

            var settings = JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(settingsFile));
            if (settings is null)
                return;

            simulatorPath = settings.SimulatorPath ?? string.Empty;
            profile = settings.Profile ?? "Balanced";

            if (settings.Visuals is null)
                return;

            foreach (var saved in settings.Visuals)
            {
                if (visuals.ContainsKey(saved.Key) && saved.Value is not null)
                {
                    visuals[saved.Key].Value = Math.Clamp(saved.Value.Value, 0, 100);
                    visuals[saved.Key].Enabled = saved.Value.Enabled;
                }
            }
        }
        catch
        {
            FooterStatus.Text = "Settings file could not be read; defaults loaded.";
        }
    }
}
