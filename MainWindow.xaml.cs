using System.IO;
using System.Text.Json;
using System.Windows;
using System.Windows.Controls;
using Microsoft.Win32;

namespace XPEnhance;
public partial class MainWindow : Window
{
    private readonly string dataDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "XPEnhance");
    private readonly string settingsFile;
    private string simulatorPath = "";
    private string profile = "Balanced";
    private readonly Dictionary<string,(double Value,bool Enabled)> visuals = new()
    {
        ["Cloud quality"]=(80,true),["Atmospheric effects"]=(75,true),["Fog realism"]=(70,true),
        ["Sunlight"]=(80,true),["Shadow quality"]=(75,true),["Water appearance"]=(75,true),
        ["Terrain detail"]=(85,true),["Ground detail"]=(85,true),["Colour grading"]=(50,true),["Sharpening"]=(40,true)
    };
    public MainWindow()
    {
        InitializeComponent(); Directory.CreateDirectory(dataDir); settingsFile=Path.Combine(dataDir,"settings.json");
        LoadSettings(); ShowPage("Dashboard");
    }
    private void Nav_Click(object sender,RoutedEventArgs e){if(sender is Button b && b.Tag is string p)ShowPage(p);}
    private void ShowPage(string page)
    {
        PageTitle.Text=page;
        PageSubtitle.Text=page switch {"Dashboard"=>"Your X-Plane 12 enhancement workspace.","Visual Settings"=>"Adjust individual realism controls.","Presets"=>"Choose a starting point for your setup.","Settings"=>"Configure your simulator installation.",_=>"This module is planned for a future release."};
        PageContent.Content=page switch {"Dashboard"=>Dashboard(),"Visual Settings"=>VisualPage(),"Presets"=>PresetsPage(),"Settings"=>SettingsPage(),_=>ComingSoon(page)};
    }
    private StackPanel Card(){return new StackPanel {Background=new System.Windows.Media.SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString("#191E26")),Margin=new Thickness(0,0,0,14)};}
    private TextBlock Txt(string text,double size=13,bool bold=false,string color="#F1F4F8")=>new(){Text=text,FontSize=size,FontWeight=bold?FontWeights.SemiBold:FontWeights.Normal,Foreground=new System.Windows.Media.SolidColorBrush((System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString(color)),Margin=new Thickness(18,8,18,8),TextWrapping=TextWrapping.Wrap};
    private Button Btn(string text,RoutedEventHandler click){var b=new Button{Content=text,HorizontalAlignment=HorizontalAlignment.Left,Margin=new Thickness(18,8,8,12)};b.Click+=click;return b;}
    private UIElement Dashboard()
    {
        var root=new StackPanel();var c=Card();c.Children.Add(Txt("XP ENHANCE · X-PLANE 12",11,true,"#75B8FF"));c.Children.Add(Txt("Make your simulator yours.",28,true));c.Children.Add(Txt("A central place to configure and manage your X-Plane enhancement setup.",14,false,"#9AA6B5"));c.Children.Add(Txt("SIMULATOR INSTALLATION",10,true,"#9AA6B5"));c.Children.Add(Txt(string.IsNullOrEmpty(simulatorPath)?"Not configured":simulatorPath));c.Children.Add(Btn("Choose X-Plane folder",(_,_)=>ChooseFolder()));c.Children.Add(Btn("Open Visual Settings",(_,_)=>ShowPage("Visual Settings")));root.Children.Add(c);
        var info=Card();info.Children.Add(Txt("ACTIVE PROFILE",10,true,"#75B8FF"));info.Children.Add(Txt(profile,22,true));info.Children.Add(Txt("Foundation build: settings and presets are saved locally. Rendering changes and addon installation are not implemented yet.",13,false,"#9AA6B5"));root.Children.Add(info);return root;
    }
    private UIElement VisualPage()
    {
        var root=new StackPanel();var intro=Card();intro.Children.Add(Txt("VISUAL CONFIGURATION",11,true,"#75B8FF"));intro.Children.Add(Txt("Tune your realism profile.",22,true));intro.Children.Add(Txt("These controls currently save profile values only; they are not yet connected to X-Plane rendering.",12,false,"#9AA6B5"));root.Children.Add(intro);
        foreach(var key in visuals.Keys.ToList())
        {
            var item=visuals[key];var row=new Grid{Margin=new Thickness(12,8,12,8)};row.ColumnDefinitions.Add(new ColumnDefinition{Width=new GridLength(180)});row.ColumnDefinitions.Add(new ColumnDefinition{Width=new GridLength(1,GridUnitType.Star)});row.ColumnDefinitions.Add(new ColumnDefinition{Width=new GridLength(55)});
            var check=new CheckBox{Content=key,IsChecked=item.Enabled,VerticalAlignment=VerticalAlignment.Center};var slider=new Slider{Minimum=0,Maximum=100,Value=item.Value,IsEnabled=item.Enabled,VerticalAlignment=VerticalAlignment.Center};var val=Txt($"{item.Value:0}%",12,false,"#9AA6B5");val.Margin=new Thickness(4);
            check.Checked+=(_,_)=>{visuals[key]=(visuals[key].Value,true);slider.IsEnabled=true;SaveSettings();};check.Unchecked+=(_,_)=>{visuals[key]=(visuals[key].Value,false);slider.IsEnabled=false;SaveSettings();};
            slider.ValueChanged+=(_,_)=>{visuals[key]=(slider.Value,check.IsChecked==true);val.Text=$"{slider.Value:0}%";SaveSettings();};
            Grid.SetColumn(check,0);Grid.SetColumn(slider,1);Grid.SetColumn(val,2);row.Children.Add(check);row.Children.Add(slider);row.Children.Add(val);
            var card=Card();card.Children.Add(row);root.Children.Add(card);
        }return root;
    }
    private UIElement PresetsPage()
    {
        var c=Card();c.Children.Add(Txt("PROFILE MANAGER",11,true,"#75B8FF"));c.Children.Add(Txt("Choose a starting point.",22,true));var combo=new ComboBox{ItemsSource=new[]{"Balanced","Ultra Realism","Performance","VR Realism","Cinematic","Custom"},SelectedItem=profile,Margin=new Thickness(18,12,18,12),Padding=new Thickness(8)};c.Children.Add(combo);c.Children.Add(Btn("Apply profile",(_,_)=>{profile=combo.SelectedItem?.ToString()??"Balanced";double v=profile switch{"Ultra Realism"=>95,"Performance"=>45,"VR Realism"=>65,"Cinematic"=>90,_=>75};foreach(var k in visuals.Keys.ToList())visuals[k]=(v,visuals[k].Enabled);SaveSettings();FooterStatus.Text=$"Saved {profile} profile · X-Plane files unchanged";ShowPage("Presets");}));return c;
    }
    private UIElement SettingsPage(){var c=Card();c.Children.Add(Txt("SIMULATOR INSTALLATION",11,true,"#75B8FF"));c.Children.Add(Txt(string.IsNullOrEmpty(simulatorPath)?"No X-Plane folder selected":simulatorPath));c.Children.Add(Btn("Browse for X-Plane 12",(_,_)=>ChooseFolder()));c.Children.Add(Txt("Settings are stored in %APPDATA%\\XPEnhance.",12,false,"#9AA6B5"));return c;}
    private UIElement ComingSoon(string page){var c=Card();c.Children.Add(Txt("PLANNED MODULE",11,true,"#75B8FF"));c.Children.Add(Txt(page,23,true));c.Children.Add(Txt("This module is on the XP Enhance roadmap and is not functional in version 0.1.0.",13,false,"#9AA6B5"));return c;}
    private void ChooseFolder(){var d=new OpenFolderDialog{Title="Select X-Plane 12 folder"};if(d.ShowDialog()!=true)return;if(!File.Exists(Path.Combine(d.FolderName,"X-Plane.exe"))&&!File.Exists(Path.Combine(d.FolderName,"X-Plane"))){MessageBox.Show("Select the folder containing X-Plane.exe.","Invalid folder",MessageBoxButton.OK,MessageBoxImage.Warning);return;}simulatorPath=d.FolderName;SaveSettings();ShowPage("Dashboard");}
    private void SaveSettings(){try{Directory.CreateDirectory(dataDir);var obj=new{simulatorPath,profile,visuals};File.WriteAllText(settingsFile,JsonSerializer.Serialize(obj,new JsonSerializerOptions{WriteIndented=true}));}catch(Exception ex){FooterStatus.Text="Could not save settings: "+ex.Message;}}
    private void LoadSettings(){try{if(!File.Exists(settingsFile))return;using var doc=JsonDocument.Parse(File.ReadAllText(settingsFile));var r=doc.RootElement;if(r.TryGetProperty("simulatorPath",out var p))simulatorPath=p.GetString()??"";if(r.TryGetProperty("profile",out var pr))profile=pr.GetString()??"Balanced";if(r.TryGetProperty("visuals",out var vs))foreach(var prop in vs.EnumerateObject())if(visuals.ContainsKey(prop.Name))visuals[prop.Name]=(prop.Value.GetProperty("Value").GetDouble(),prop.Value.GetProperty("Enabled").GetBoolean());}catch{FooterStatus.Text="Settings file could not be read; defaults loaded.";}}
}