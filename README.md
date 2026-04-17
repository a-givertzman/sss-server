# sss-server

## General

Программное обеспечение в качестве грузового компьютера предназначено для создания и хранения грузового плана судна, расчета характеристик посадки, остойчивости, перерезывающих сил и изгибающих моментов на тихой воде, а также сравнения их с допустимыми значениями.

## Dependencies

## Configuration

## Communication Interface

Communication between SSS-Client and SSS-Server based on events over TCP/IP

- [API Reference](src/server/README.md)
- 
For create and open documentation: cargo doc --no-deps --open
For full test run: cargo test full --release

- sss tag: release_2_0.0.15
- sss-captain-report tag: release_2_0.0.7
