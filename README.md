<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->

<a id="readme-top"></a>

<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->

<!-- PROJECT LOGO -->
<br />
<div align="center">
<h3 align="center">smaps-explorer</h3>

  <p align="center">
    A simple tui to visualize the /proc/smaps of your running linux process written in rust.
    <br />
  </p>
</div>

<img src="static/demo.gif" alt="demo" style="display: block; max-width: 100%; height: auto; border: none;">

<!-- GETTING STARTED -->

## Getting Started

If you are running on a systemd based linux distro you may simply run `make run` to get up and running with an example.

### Installation

1. Clone the repo
   ```sh
   git clone https://github.com/github_username/smaps-explorer.git
   ```
2. Run
   ```sh
   cargo run 
   ```

<!-- USAGE EXAMPLES -->

## Usage
```sh
Usage: smaps-explorer [OPTIONS] <PID>

Arguments:
  <PID>  or '-' for stdin.

Options:
  -d, --debug
  -h, --help   Print help
```

<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<!-- Known Issues -->

## Known Issues
- Help screen text does not wrap to next line. 

<!-- LICENSE -->

## License

Distributed under the MIT License. See `LICENSE` for more information.

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->