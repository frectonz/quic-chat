package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"io"
	"os"

	"github.com/quic-go/quic-go"
	"github.com/vmihailenco/msgpack/v5"
)

const addr = "localhost:5000"

type PostMessage struct {
	Post []string
}

type GetAllResponse struct {
	Messages []string
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("expected 'post', 'get-all' or 'clear' subcommands")
		os.Exit(1)
	}

	tlsConf := &tls.Config{
		InsecureSkipVerify: true,
	}
	conn, err := quic.DialAddr(context.Background(), addr, tlsConf, nil)
	if err != nil {
		panic(err)
	}

	stream, err := conn.AcceptStream(context.Background())
	if err != nil {
		panic(err)
	}
	var buf []byte

	// 1. read hello message
	buf, err = read(stream)

	var hello string
	err = msgpack.Unmarshal(buf, &hello)
	if err != nil {
		panic(err)
	}
	fmt.Println("client: got", hello)
	assert(hello == "Hello")

	switch os.Args[1] {
	case "post":
		if len(os.Args) != 3 {
			fmt.Println("'post' subcommand needs a message")
			os.Exit(1)
		}

		message := PostMessage{[]string{os.Args[2]}}
		buf, err = msgpack.Marshal(&message)
		if err != nil {
			panic(err)
		}

		err = write(stream, buf)
		if err != nil {
			panic(err)
		}

		buf, err = read(stream)
		if err != nil {
			panic(err)
		}

		var response string
		err = msgpack.Unmarshal(buf, &response)
		if err != nil {
			panic(err)
		}
		fmt.Println("client: got", response)

		assert(response == "OK")

	case "clear":
		message := "Clear"
		buf, err = msgpack.Marshal(&message)
		if err != nil {
			panic(err)
		}

		err = write(stream, buf)
		if err != nil {
			panic(err)
		}

		buf, err = read(stream)
		if err != nil {
			panic(err)
		}

		var response string
		err = msgpack.Unmarshal(buf, &response)
		if err != nil {
			panic(err)
		}
		fmt.Println("client: got", response)

		assert(response == "OK")

	case "get-all":
		message := "GetAll"
		buf, err = msgpack.Marshal(&message)
		if err != nil {
			panic(err)
		}

		err = write(stream, buf)
		if err != nil {
			panic(err)
		}

		buf, err = read(stream)
		if err != nil {
			panic(err)
		}

		var response GetAllResponse
		err = msgpack.Unmarshal(buf, &response)
		if err != nil {
			panic(err)
		}
		fmt.Println("client: got", response)

	default:
		fmt.Println("expected 'post', 'get-all' or 'clear' subcommands")
		os.Exit(1)
	}

}

func assert(check bool) {
	if !check {
		panic("that wasn't supposed to happen")
	}
}

func read(stream quic.Stream) ([]byte, error) {
	buf := make([]byte, 512)
	n_bytes, err := stream.Read(buf)
	fmt.Println("client: read", n_bytes, "bytes")

	if err == io.EOF {
		return buf, nil
	}

	if err != nil {
		return nil, err
	}

	return buf, nil
}

func write(stream quic.Stream, buf []byte) error {
	n_bytes, err := stream.Write(buf)
	fmt.Println("client: wrote", n_bytes, "bytes")
	if err != nil {
		return err
	}

	return nil
}
